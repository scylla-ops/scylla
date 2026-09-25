use super::TRIGGER_RUNNER_APP_NAME;
use crate::application::{
    AppRepository, DispatchOutcome, DispatchUseCases, PipelineRepository, PipelineUseCases,
    ProjectRepository, TriggerRepository,
};
use crate::domain::caller::CallerContext;
use crate::domain::clock;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, OrganizationId, TriggerId};
use crate::domain::job::Job;
use crate::domain::job::JobOrigin;
use crate::domain::permission::Permission;
use crate::domain::trigger::Trigger;
use crate::domain::trigger::{TriggerInputSource, TriggerSource};
use async_trait::async_trait;
use derive_more::Constructor;
use scylla_extension::{
    Access, Authorized, Command, Committed, Describe, Persist, Prepare, Prepared, Run,
};
use std::sync::Arc;
use tracing::{instrument, warn};

#[async_trait]
pub trait TriggerFiring: Send + Sync {
    async fn fire(
        &self,
        trigger_id: &TriggerId,
        payload: Option<&serde_json::Value>,
        delivery_id: Option<&str>,
    ) -> DomainResult<Job>;
}

/// Every fire runs as the org's trigger-runner App through `run_with_inputs`: one `RunPipeline` check, normal dispatch.
#[derive(Constructor)]
pub struct TriggerFireUseCases {
    trigger_repo: Arc<dyn TriggerRepository>,
    pipeline_repo: Arc<dyn PipelineRepository>,
    project_repo: Arc<dyn ProjectRepository>,
    app_repo: Arc<dyn AppRepository>,
    pipeline_uc: Arc<PipelineUseCases>,
    dispatch_uc: Arc<DispatchUseCases>,
}

impl TriggerFireUseCases {
    #[instrument(skip_all, fields(trigger_id = %trigger_id))]
    pub async fn fire(
        &self,
        trigger_id: &TriggerId,
        payload: Option<&serde_json::Value>,
        delivery_id: Option<&str>,
    ) -> DomainResult<Job> {
        let trigger = self.trigger_repo.find_by_id(trigger_id).await?;
        self.fire_loaded(trigger, payload, delivery_id).await
    }

    async fn fire_loaded(
        &self,
        mut trigger: Trigger,
        payload: Option<&serde_json::Value>,
        delivery_id: Option<&str>,
    ) -> DomainResult<Job> {
        if !trigger.is_enabled() {
            return Err(DomainError::business_rule("trigger is disabled"));
        }

        let outcome = self.run_and_dispatch(&trigger, payload, delivery_id).await;

        // Best-effort: a status-write failure must not mask the run outcome.
        trigger.mark_fired(clock::now(), if outcome.is_ok() { "ok" } else { "error" });
        if let Err(e) = self.trigger_repo.update(&trigger).await {
            warn!(trigger_id = %trigger.id(), error = %e, "failed to record trigger fire status");
        }
        outcome
    }

    async fn run_and_dispatch(
        &self,
        trigger: &Trigger,
        payload: Option<&serde_json::Value>,
        delivery_id: Option<&str>,
    ) -> DomainResult<Job> {
        let pipeline = self.pipeline_repo.find_by_id(trigger.pipeline_id()).await?;
        let project = self.project_repo.find_by_id(pipeline.project_id()).await?;
        let runner = self.runner_app(project.organization_id()).await?;
        let caller = CallerContext::App(runner);

        // The origin is the trigger, not the runner App it executes as.
        let origin = match trigger.source() {
            TriggerSource::Cron(_) => JobOrigin::Cron {
                trigger_id: trigger.id().clone(),
            },
            TriggerSource::Webhook(_) => JobOrigin::Webhook {
                trigger_id: trigger.id().clone(),
                delivery_id: delivery_id.map(str::to_owned),
            },
        };

        let inputs = resolve_inputs(trigger, payload);
        let (job, dispatch) = self
            .pipeline_uc
            .run_with_inputs(&caller, trigger.pipeline_id(), &inputs, origin)
            .await?;

        if let DispatchOutcome::Dispatched(app_id) = self
            .dispatch_uc
            .dispatch_job(trigger.pipeline_id(), &dispatch)
            .await?
        {
            self.pipeline_uc.assign_agent(job.id(), &app_id).await?;
        }
        Ok(job)
    }

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

/// The fire runs as the trigger-runner App, as a scheduled one does; the caller's
/// `runPipeline` on the trigger is the authorize stage.
#[derive(Debug)]
pub struct FireTriggerNow {
    pub id: TriggerId,
}

impl Describe for FireTriggerNow {
    fn access(&self) -> Access {
        Access::Requires(Permission::RunTriggerPipeline(self.id.clone()))
    }
}

impl Command for FireTriggerNow {
    type Staged = Trigger;
    type Committed = Job;
}

#[async_trait]
impl Run<Prepare<FireTriggerNow>> for TriggerFireUseCases {
    async fn run(
        &self,
        input: Authorized<FireTriggerNow>,
    ) -> DomainResult<Prepared<FireTriggerNow>> {
        let trigger = self.trigger_repo.find_by_id(&input.command().id).await?;
        Ok(input.prepared(trigger))
    }
}

#[async_trait]
impl Run<Persist<FireTriggerNow>> for TriggerFireUseCases {
    async fn run(
        &self,
        input: Prepared<FireTriggerNow>,
    ) -> DomainResult<Committed<FireTriggerNow>> {
        input
            .commit(async |trigger| self.fire_loaded(trigger, None, None).await)
            .await
    }
}

#[async_trait]
impl TriggerFiring for TriggerFireUseCases {
    async fn fire(
        &self,
        trigger_id: &TriggerId,
        payload: Option<&serde_json::Value>,
        delivery_id: Option<&str>,
    ) -> DomainResult<Job> {
        // The inherent method, not the trait one: no recursion.
        TriggerFireUseCases::fire(self, trigger_id, payload, delivery_id).await
    }
}

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

fn json_value_to_env(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::PipelineId;
    use crate::domain::pipeline::EnvKey;
    use crate::domain::trigger::{CronSpec, TriggerInput, TriggerName, TriggerSource, WebhookSpec};
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
        assert_eq!(
            resolved,
            vec![("RUN_MODE".to_string(), "nightly".to_string())]
        );
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
