use super::{RecordTriggerFire, TRIGGER_RUNNER_APP_NAME, TriggerUseCases};
use crate::application::pipeline::RunPipelineWithInputs;
use crate::application::{PipelineUseCases, TriggerRepository};
use crate::domain::caller::{CallerContext, ServiceIdentity};
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
    Access, Actions, Authorized, Command, Committed, Describe, Persist, Prepare, Prepared, Run,
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

/// The fire of the cron scheduler, the webhook ingress and `FireTriggerNow`. It runs as the
/// server and sends two commands: `RunPipelineWithInputs` as the organization's trigger-runner
/// App, then `RecordTriggerFire` as the trigger firer service, best-effort.
#[derive(Constructor)]
pub struct TriggerFirer {
    actions: Arc<Actions>,
    triggers: Arc<TriggerUseCases>,
    pipelines: Arc<PipelineUseCases>,
}

impl TriggerFirer {
    async fn run(
        &self,
        trigger: &Trigger,
        payload: Option<&serde_json::Value>,
        delivery_id: Option<&str>,
    ) -> DomainResult<Job> {
        let pipeline = self
            .triggers
            .pipeline_repo
            .find_by_id(trigger.pipeline_id())
            .await?;
        let project = self
            .triggers
            .project_repo
            .find_by_id(pipeline.project_id())
            .await?;
        let runner = self.runner_app(project.organization_id()).await?;

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
        let run = RunPipelineWithInputs {
            id: trigger.pipeline_id().clone(),
            inputs: resolve_inputs(trigger, payload),
            origin,
        };
        self.actions
            .run(&*self.pipelines, &CallerContext::App(runner), run)
            .await
    }

    async fn runner_app(&self, organization_id: &OrganizationId) -> DomainResult<AppId> {
        self.triggers
            .app_repo
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
impl TriggerFiring for TriggerFirer {
    #[instrument(skip_all, fields(trigger_id = %trigger_id))]
    async fn fire(
        &self,
        trigger_id: &TriggerId,
        payload: Option<&serde_json::Value>,
        delivery_id: Option<&str>,
    ) -> DomainResult<Job> {
        let trigger = self.triggers.trigger_repo.find_by_id(trigger_id).await?;
        if !trigger.is_enabled() {
            return Err(DomainError::business_rule("trigger is disabled"));
        }

        let outcome = self.run(&trigger, payload, delivery_id).await;

        // Best-effort: a status-write failure must not mask the run outcome.
        let caller = CallerContext::Service(ServiceIdentity::trigger_firer());
        let record = RecordTriggerFire {
            trigger,
            status: if outcome.is_ok() { "ok" } else { "error" },
        };
        if let Err(e) = self.actions.run(&*self.triggers, &caller, record).await {
            warn!(trigger_id = %trigger_id, error = %e, "failed to record trigger fire status");
        }
        outcome
    }
}

/// The stage runner of `FireTriggerNow`: the fire goes through `firing`, as a scheduled one does.
#[derive(Constructor)]
pub struct TriggerFireUseCases {
    trigger_repo: Arc<dyn TriggerRepository>,
    firing: Arc<dyn TriggerFiring>,
}

/// The caller's `runPipeline` on the trigger is the authorize stage; the fire then runs as the
/// trigger-runner App.
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
            .commit(async |trigger| self.firing.fire(trigger.id(), None, None).await)
            .await
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
