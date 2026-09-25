pub mod commands;
pub mod delivery;
pub mod fire;
pub mod queries;
pub mod repository;
pub mod schedule;
pub mod scheduler;
pub mod webhook;

pub use commands::{
    ClaimDueTriggers, CreateTrigger, CreatedTrigger, DeleteTrigger, NewTrigger, RecordTriggerFire,
    ScheduleCronTriggers, SetTriggerEnabled, UpdateTrigger,
};
pub use delivery::TriggerDeliveryRepository;
pub use fire::{FireTriggerNow, TriggerFireUseCases, TriggerFirer, TriggerFiring};
pub use queries::{GetTrigger, ListPipelineTriggers, ResolveTriggerRun, TriggerRun};
pub use repository::TriggerRepository;
pub use schedule::{CronSchedule, next_fire_time};
pub use scheduler::TriggerCronScheduler;
pub use webhook::{
    DEFAULT_SIGNATURE_HEADER, IngestOutcome, IngestWebhook, WebhookIngressUseCases,
    verify_signature,
};

use crate::application::{
    AppRepository, HashService, PipelineRepository, ProjectRepository, SecretCipher,
};
use crate::domain::app::{App, AppCredential, AppName, AppSecretLabel};
use crate::domain::clock;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, OrganizationId};
use crate::domain::role::RoleName;
use crate::domain::trigger::Trigger;
use derive_more::Constructor;
use scylla_auth::authz::{
    Grant, ORGANIZATION_TRIGGER_RUNNER_ROLE, PolicyControl, Principal, Scope,
};
use std::sync::Arc;

/// Holds `organization-trigger-runner` (only `runPipeline`); used in-process, never via a token.
pub(crate) const TRIGGER_RUNNER_APP_NAME: &str = "trigger-runner";
const RUNNER_SECRET_LABEL: &str = "default";

/// The trigger aggregate's stage runners, one block per action in `commands.rs` and
/// `queries.rs`. It has no public method; `Actions::run` drives it, also for the reads and the
/// writes of the cron scheduler and of the trigger firer.
#[allow(clippy::too_many_arguments)]
#[derive(Constructor)]
pub struct TriggerUseCases {
    pub(super) trigger_repo: Arc<dyn TriggerRepository>,
    pub(super) pipeline_repo: Arc<dyn PipelineRepository>,
    pub(super) project_repo: Arc<dyn ProjectRepository>,
    pub(super) app_repo: Arc<dyn AppRepository>,
    pub(super) hash_service: Arc<dyn HashService>,
    pub(super) policy_control: Arc<dyn PolicyControl>,
    /// Reversible: HMAC verification needs the plaintext back.
    pub(super) cipher: Arc<dyn SecretCipher>,
    pub(super) schedule: Arc<dyn CronSchedule>,
}

impl TriggerUseCases {
    fn schedule_next(&self, trigger: &mut Trigger) -> DomainResult<()> {
        let next = next_fire_time(trigger, &*self.schedule, clock::now())?;
        trigger.set_next_fire_at(next);
        Ok(())
    }

    async fn runner_app(&self, organization_id: &OrganizationId) -> DomainResult<Option<AppId>> {
        Ok(self
            .app_repo
            .list_by_organization(organization_id)
            .await?
            .into_iter()
            .find(|app| app.name().to_string() == TRIGGER_RUNNER_APP_NAME)
            .map(|app| app.id().clone()))
    }

    async fn trigger_runner(&self, trigger: &Trigger) -> DomainResult<AppId> {
        let pipeline = self.pipeline_repo.find_by_id(trigger.pipeline_id()).await?;
        let project = self.project_repo.find_by_id(pipeline.project_id()).await?;
        self.runner_app(project.organization_id())
            .await?
            .ok_or_else(|| {
                DomainError::internal("trigger-runner App is not provisioned for this organization")
            })
    }

    async fn ensure_runner_app(&self, organization_id: &OrganizationId) -> DomainResult<()> {
        if self.runner_app(organization_id).await?.is_some() {
            return Ok(());
        }

        let app = App::create(
            organization_id.clone(),
            AppName::new(TRIGGER_RUNNER_APP_NAME)?,
        );
        let secret = crate::application::app::mint_app_secret();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let credential = AppCredential::create(
            app.id().clone(),
            AppSecretLabel::new(RUNNER_SECRET_LABEL)?,
            secret_hash,
        );
        let grant = Grant::new(
            Principal::App(app.id().clone()),
            RoleName::new(ORGANIZATION_TRIGGER_RUNNER_ROLE)?,
            Scope::Organization(organization_id.clone()),
        );

        match self.app_repo.provision(&app, &credential, &grant).await {
            Ok(()) => {
                self.policy_control.reload().await?;
                Ok(())
            }
            // Lost the race to a concurrent first-create.
            Err(DomainError::Conflict(_)) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests;
