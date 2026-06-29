use super::use_case::TRIGGER_RUNNER_APP_NAME;
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::application::caller::CallerContext;
use crate::application::{
    AppRepository, DispatchOutcome, DispatchUseCases, JobRepository, PermissionService,
    PipelineRepository, PipelineUseCases, ProjectRepository, TriggerRepository,
};
use crate::domain::clock;
use crate::domain::entities::{AppId, Job, OrganizationId, Trigger, TriggerId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::job::JobOrigin;
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::trigger::{TriggerInputSource, TriggerSource};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::instrument;

/// The act of firing a trigger by id, decoupled from the concrete use-case so
/// background drivers (the cron scheduler, later the webhook ingress) depend on
/// behaviour, not on `TriggerFireUseCases`' seven generic parameters.
#[async_trait]
pub trait TriggerFiring: Send + Sync {
    /// Fire `trigger_id`; `payload` carries a webhook body (`None` for cron /
    /// manual), `delivery_id` the sender's idempotency key for webhook fires
    /// (`None` otherwise). Mints + dispatches a job and records the outcome on the
    /// trigger.
    async fn fire(
        &self,
        trigger_id: &TriggerId,
        payload: Option<&serde_json::Value>,
        delivery_id: Option<&str>,
    ) -> DomainResult<Job>;
}

/// Fires triggers — the single path that turns a due cron tick, a manual
/// "run now", or (later) a verified webhook into an actual run. Every fire runs
/// as the org's trigger-runner App through the unchanged
/// [`PipelineUseCases::run_with_inputs`], so it inherits the one `RunPipeline`
/// Cedar check and the normal agent dispatch + pending-retry safety net.
pub struct TriggerFireUseCases<T, P, PR, A, J, PS, W>
where
    T: TriggerRepository,
    P: PipelineRepository,
    PR: ProjectRepository,
    A: AppRepository,
    J: JobRepository,
    PS: PermissionService,
    W: AgentDispatch,
{
    trigger_repo: Arc<T>,
    pipeline_repo: Arc<P>,
    project_repo: Arc<PR>,
    app_repo: Arc<A>,
    pipeline_uc: Arc<PipelineUseCases<P, PR, J, PS>>,
    dispatch_uc: Arc<DispatchUseCases<W, PS>>,
    /// Authorizes the caller on the manual `fire_now` path (`RunPipeline` on the
    /// trigger's pipeline). The background `fire` (cron / webhook) has no caller
    /// and does not use it.
    permission_service: Arc<PS>,
}

impl<T, P, PR, A, J, PS, W> TriggerFireUseCases<T, P, PR, A, J, PS, W>
where
    T: TriggerRepository,
    P: PipelineRepository,
    PR: ProjectRepository,
    A: AppRepository,
    J: JobRepository,
    PS: PermissionService,
    W: AgentDispatch,
{
    #[must_use]
    pub fn new(
        trigger_repo: Arc<T>,
        pipeline_repo: Arc<P>,
        project_repo: Arc<PR>,
        app_repo: Arc<A>,
        pipeline_uc: Arc<PipelineUseCases<P, PR, J, PS>>,
        dispatch_uc: Arc<DispatchUseCases<W, PS>>,
        permission_service: Arc<PS>,
    ) -> Self {
        Self {
            trigger_repo,
            pipeline_repo,
            project_repo,
            app_repo,
            pipeline_uc,
            dispatch_uc,
            permission_service,
        }
    }

    /// Fire the trigger `trigger_id` now on behalf of `caller` (the manual "run
    /// now" path). Unlike the background [`fire`](Self::fire) — driven by the cron
    /// scheduler / webhook ingress with no human principal — this authorizes the
    /// caller first: a manual fire runs the pipeline, so it is gated by
    /// `RunPipeline` on the trigger's pipeline (the run itself still executes as
    /// the org's trigger-runner App). No webhook payload, so only literal inputs
    /// apply.
    #[instrument(skip(self, caller), fields(trigger_id = %trigger_id))]
    pub async fn fire_now(
        &self,
        caller: &CallerContext,
        trigger_id: &TriggerId,
    ) -> DomainResult<Job> {
        let trigger = self.trigger_repo.find_by_id(trigger_id).await?;
        self.permission_service
            .check(caller, Permission::RunPipeline(trigger.pipeline_id().clone()))
            .await?;
        self.fire(trigger_id, None, None).await
    }

    /// Fire the trigger `trigger_id` now. `payload` is the webhook body when the
    /// fire is webhook-driven (used to resolve json-pointer inputs); `None` for
    /// cron / manual fires, where only literal inputs apply. `delivery_id` is the
    /// webhook delivery key, recorded in the job's `Webhook` origin. Mints +
    /// dispatches a job exactly like a manual run and records the fire outcome.
    #[instrument(skip(self, payload, delivery_id), fields(trigger_id = %trigger_id))]
    pub async fn fire(
        &self,
        trigger_id: &TriggerId,
        payload: Option<&serde_json::Value>,
        delivery_id: Option<&str>,
    ) -> DomainResult<Job> {
        let mut trigger = self.trigger_repo.find_by_id(trigger_id).await?;
        if !trigger.is_enabled() {
            return Err(DomainError::business_rule("trigger is disabled"));
        }

        // The run fires as the org's trigger-runner App (resolved pipeline →
        // project → org → App), so the single RunPipeline check is satisfied by
        // its org-scoped grant.
        let pipeline = self.pipeline_repo.find_by_id(trigger.pipeline_id()).await?;
        let project = self.project_repo.find_by_id(pipeline.project_id()).await?;
        let runner = self.runner_app(project.organization_id()).await?;
        let caller = CallerContext::App(runner);

        // The run's origin is the trigger (cron / webhook), NOT the runner App it
        // executes as. Webhook carries the delivery id when the sender provided one.
        let origin = match trigger.source() {
            TriggerSource::Cron(_) => JobOrigin::Cron {
                trigger_id: trigger_id.clone(),
            },
            TriggerSource::Webhook(_) => JobOrigin::Webhook {
                trigger_id: trigger_id.clone(),
                delivery_id: delivery_id.map(str::to_owned),
            },
        };

        let inputs = resolve_inputs(&trigger, payload);
        let (job, dispatch) = self
            .pipeline_uc
            .run_with_inputs(&caller, trigger.pipeline_id(), &inputs, origin)
            .await?;

        // Best-effort placement: if no agent is connected/authorized the job
        // stays pending and the PendingJobScheduler retries it later.
        if let DispatchOutcome::Dispatched(app_id) =
            self.dispatch_uc.dispatch_job(trigger.pipeline_id(), &dispatch).await?
        {
            self.pipeline_uc.assign_agent(job.id(), &app_id).await?;
        }

        trigger.mark_fired(clock::now(), "ok");
        self.trigger_repo.update(&trigger).await?;
        Ok(job)
    }

    /// The org's trigger-runner App id (provisioned at trigger-create time).
    async fn runner_app(&self, organization_id: &OrganizationId) -> DomainResult<AppId> {
        self.app_repo
            .list_by_organization(organization_id)
            .await?
            .into_iter()
            .find(|app| app.name().to_string() == TRIGGER_RUNNER_APP_NAME)
            .map(|app| app.id().clone())
            .ok_or_else(|| {
                DomainError::internal("trigger-runner App is not provisioned for this organization")
            })
    }
}

#[async_trait]
impl<T, P, PR, A, J, PS, W> TriggerFiring for TriggerFireUseCases<T, P, PR, A, J, PS, W>
where
    T: TriggerRepository + Send + Sync,
    P: PipelineRepository + Send + Sync,
    PR: ProjectRepository + Send + Sync,
    A: AppRepository + Send + Sync,
    J: JobRepository + Send + Sync,
    PS: PermissionService + Send + Sync,
    W: AgentDispatch + Send + Sync,
{
    async fn fire(
        &self,
        trigger_id: &TriggerId,
        payload: Option<&serde_json::Value>,
        delivery_id: Option<&str>,
    ) -> DomainResult<Job> {
        // Delegate to the inherent method (preferred in resolution, so no recursion).
        TriggerFireUseCases::fire(self, trigger_id, payload, delivery_id).await
    }
}

/// Resolve a trigger's declared inputs into concrete `(key, value)` env pairs.
/// Literals pass through; json-pointer inputs are extracted from `payload` (RFC
/// 6901) and silently skipped when there is no payload (cron / manual fire) or
/// the pointer doesn't resolve.
fn resolve_inputs(trigger: &Trigger, payload: Option<&serde_json::Value>) -> Vec<(String, String)> {
    trigger
        .inputs()
        .iter()
        .filter_map(|input| match input.source() {
            TriggerInputSource::Literal(value) => Some((input.key().to_string(), value.clone())),
            TriggerInputSource::JsonPointer(pointer) => payload
                .and_then(|body| body.pointer(pointer))
                .map(|v| (input.key().to_string(), json_value_to_env(v))),
        })
        .collect()
}

/// Render a JSON value as an env string: strings pass through unquoted, scalars
/// use their JSON form (`123`, `true`), composites their compact JSON.
fn json_value_to_env(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::PipelineId;
    use crate::domain::value_objects::pipeline::EnvKey;
    use crate::domain::value_objects::trigger::{
        CronSpec, TriggerInput, TriggerName, TriggerSource, WebhookSpec,
    };
    use serde_json::json;

    fn webhook_trigger(inputs: Vec<TriggerInput>) -> Trigger {
        Trigger::create(
            PipelineId::new("p"),
            TriggerName::new("on-push").unwrap(),
            TriggerSource::Webhook(WebhookSpec::new(None).unwrap()),
            inputs,
        )
        .unwrap()
    }

    fn key(k: &str) -> EnvKey {
        EnvKey::new(k).unwrap()
    }

    #[test]
    fn literal_inputs_resolve_without_payload() {
        let trigger = Trigger::create(
            PipelineId::new("p"),
            TriggerName::new("nightly").unwrap(),
            TriggerSource::Cron(CronSpec::new("0 9 * * *").unwrap()),
            vec![TriggerInput::literal(key("RUN_MODE"), "nightly")],
        )
        .unwrap();
        let resolved = resolve_inputs(&trigger, None);
        assert_eq!(resolved, vec![("RUN_MODE".to_string(), "nightly".to_string())]);
    }

    #[test]
    fn json_pointer_inputs_resolve_from_payload() {
        let trigger = webhook_trigger(vec![
            TriggerInput::json_pointer(key("GIT_COMMIT"), "/after").unwrap(),
            TriggerInput::json_pointer(key("REPO"), "/repository/name").unwrap(),
        ]);
        let payload = json!({ "after": "abc123", "repository": { "name": "scylla" } });
        let resolved = resolve_inputs(&trigger, Some(&payload));
        assert!(resolved.contains(&("GIT_COMMIT".to_string(), "abc123".to_string())));
        assert!(resolved.contains(&("REPO".to_string(), "scylla".to_string())));
    }

    #[test]
    fn json_pointer_skipped_without_payload_or_when_absent() {
        let trigger = webhook_trigger(vec![
            TriggerInput::json_pointer(key("GIT_COMMIT"), "/after").unwrap(),
        ]);
        assert!(resolve_inputs(&trigger, None).is_empty());
        let payload = json!({ "other": 1 });
        assert!(resolve_inputs(&trigger, Some(&payload)).is_empty());
    }

    #[test]
    fn non_string_json_values_are_coerced() {
        let trigger = webhook_trigger(vec![
            TriggerInput::json_pointer(key("COUNT"), "/n").unwrap(),
            TriggerInput::json_pointer(key("FLAG"), "/ok").unwrap(),
        ]);
        let payload = json!({ "n": 42, "ok": true });
        let resolved = resolve_inputs(&trigger, Some(&payload));
        assert!(resolved.contains(&("COUNT".to_string(), "42".to_string())));
        assert!(resolved.contains(&("FLAG".to_string(), "true".to_string())));
    }
}
