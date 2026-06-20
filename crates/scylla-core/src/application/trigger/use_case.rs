use crate::application::caller::CallerContext;
use crate::application::{
    AppRepository, Grant, HashService, PermissionService, PipelineRepository, PolicyControl,
    Principal, ProjectRepository, Scope, TriggerRepository,
};
use crate::domain::entities::{App, AppCredential, OrganizationId, PipelineId, Trigger, TriggerId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::app::{AppName, AppSecret, AppSecretLabel};
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::trigger::{TriggerInput, TriggerName, TriggerSource};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

/// Name of the per-organization machine App that pipeline runs fire as. One is
/// lazily provisioned per org on first trigger creation; it holds a single
/// `runPipeline` permission grant at org scope (NOT the agent role, which only
/// confers `executeJob`). It is used in-process as `CallerContext::App`, never
/// via a token, so its credential is provisioned only to keep the App invariant.
pub(crate) const TRIGGER_RUNNER_APP_NAME: &str = "trigger-runner";
const RUNNER_SECRET_LABEL: &str = "default";
/// Must equal `Permission::RunPipeline(_).key()` — asserted in tests.
const RUN_PIPELINE_PERMISSION_KEY: &str = "runPipeline";

/// Manage a pipeline's triggers. Every method is Cedar-gated by
/// `ManageTriggers`; create/update additionally require `RunPipeline` on the
/// caller (anti-escalation: you cannot set up a trigger that runs a pipeline you
/// could not run yourself). Firing is handled separately (the firing engine /
/// FireTrigger), all converging on the unchanged `PipelineUseCases::run`.
#[derive(Constructor)]
pub struct TriggerUseCases<T, P, PR, A, H, PC, PS>
where
    T: TriggerRepository,
    P: PipelineRepository,
    PR: ProjectRepository,
    A: AppRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
{
    trigger_repo: Arc<T>,
    pipeline_repo: Arc<P>,
    project_repo: Arc<PR>,
    app_repo: Arc<A>,
    hash_service: Arc<H>,
    policy_control: Arc<PC>,
    permission_service: Arc<PS>,
}

impl<T, P, PR, A, H, PC, PS> TriggerUseCases<T, P, PR, A, H, PC, PS>
where
    T: TriggerRepository,
    P: PipelineRepository,
    PR: ProjectRepository,
    A: AppRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
{
    #[instrument(skip(self, caller, source, inputs), fields(pipeline_id = %pipeline_id, name = %name))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        pipeline_id: PipelineId,
        name: TriggerName,
        source: TriggerSource,
        inputs: Vec<TriggerInput>,
    ) -> DomainResult<Trigger> {
        self.permission_service
            .check(caller, Permission::ManageTriggers(pipeline_id.clone()))
            .await?;
        // Anti-escalation: managing triggers must not launder run rights — the
        // creator must also be allowed to run the pipeline directly.
        self.permission_service
            .check(caller, Permission::RunPipeline(pipeline_id.clone()))
            .await?;

        // Resolve the owning org (pipeline → project → org) and make sure its
        // trigger-runner App exists before persisting the trigger.
        let pipeline = self.pipeline_repo.find_by_id(&pipeline_id).await?;
        let project = self.project_repo.find_by_id(pipeline.project_id()).await?;
        self.ensure_runner_app(project.organization_id()).await?;

        let trigger = Trigger::create(pipeline_id, name, source, inputs)?;
        self.trigger_repo.create(&trigger).await
    }

    #[instrument(skip(self, caller), fields(trigger_id = %trigger_id))]
    pub async fn get(&self, caller: &CallerContext, trigger_id: &TriggerId) -> DomainResult<Trigger> {
        let trigger = self.trigger_repo.find_by_id(trigger_id).await?;
        self.permission_service
            .check(caller, Permission::ManageTriggers(trigger.pipeline_id().clone()))
            .await?;
        Ok(trigger)
    }

    #[instrument(skip(self, caller), fields(pipeline_id = %pipeline_id))]
    pub async fn list_by_pipeline(
        &self,
        caller: &CallerContext,
        pipeline_id: &PipelineId,
    ) -> DomainResult<Vec<Trigger>> {
        self.permission_service
            .check(caller, Permission::ManageTriggers(pipeline_id.clone()))
            .await?;
        self.trigger_repo.list_by_pipeline(pipeline_id).await
    }

    #[instrument(skip(self, caller, source, inputs), fields(trigger_id = %trigger_id))]
    pub async fn update(
        &self,
        caller: &CallerContext,
        trigger_id: &TriggerId,
        name: TriggerName,
        source: TriggerSource,
        inputs: Vec<TriggerInput>,
    ) -> DomainResult<Trigger> {
        let mut trigger = self.trigger_repo.find_by_id(trigger_id).await?;
        self.permission_service
            .check(caller, Permission::ManageTriggers(trigger.pipeline_id().clone()))
            .await?;
        // Same anti-escalation fence as create: editing what a trigger fires
        // (source/inputs) requires the right to run the pipeline.
        self.permission_service
            .check(caller, Permission::RunPipeline(trigger.pipeline_id().clone()))
            .await?;
        trigger.update(name, source, inputs)?;
        self.trigger_repo.update(&trigger).await
    }

    #[instrument(skip(self, caller), fields(trigger_id = %trigger_id, enabled))]
    pub async fn set_enabled(
        &self,
        caller: &CallerContext,
        trigger_id: &TriggerId,
        enabled: bool,
    ) -> DomainResult<Trigger> {
        let mut trigger = self.trigger_repo.find_by_id(trigger_id).await?;
        self.permission_service
            .check(caller, Permission::ManageTriggers(trigger.pipeline_id().clone()))
            .await?;
        trigger.set_enabled(enabled);
        self.trigger_repo.update(&trigger).await
    }

    #[instrument(skip(self, caller), fields(trigger_id = %trigger_id))]
    pub async fn delete(&self, caller: &CallerContext, trigger_id: &TriggerId) -> DomainResult<()> {
        let trigger = self.trigger_repo.find_by_id(trigger_id).await?;
        self.permission_service
            .check(caller, Permission::ManageTriggers(trigger.pipeline_id().clone()))
            .await?;
        self.trigger_repo.delete(trigger_id).await
    }

    /// Ensure the org has its trigger-runner App (App + credential + a direct
    /// `runPipeline` grant at org scope), provisioned once and reused by every
    /// trigger in the org. Idempotent: a concurrent first-create that loses the
    /// `UNIQUE(org, name)` race is treated as already-provisioned.
    async fn ensure_runner_app(&self, organization_id: &OrganizationId) -> DomainResult<()> {
        let existing = self.app_repo.list_by_organization(organization_id).await?;
        if existing
            .iter()
            .any(|app| app.name().to_string() == TRIGGER_RUNNER_APP_NAME)
        {
            return Ok(());
        }

        let app = App::create(
            organization_id.clone(),
            AppName::new(TRIGGER_RUNNER_APP_NAME)?,
        );
        let secret = AppSecret::generate();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let credential = AppCredential::create(
            app.id().clone(),
            AppSecretLabel::new(RUNNER_SECRET_LABEL)?,
            secret_hash,
        );
        let grant = Grant::with_permission(
            Principal::App(app.id().clone()),
            RUN_PIPELINE_PERMISSION_KEY,
            Scope::Organization(organization_id.clone()),
        );

        match self.app_repo.provision(&app, &credential, &grant).await {
            Ok(()) => {
                self.policy_control.reload().await?;
                Ok(())
            }
            // Lost the race to a concurrent first-create — the runner now exists.
            Err(DomainError::Conflict(_)) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::value_objects::permission::Permission;
    use crate::domain::entities::PipelineId;

    #[test]
    fn run_pipeline_key_matches_permission_enum() {
        // The literal used for the runner's direct grant must equal the canonical
        // Permission key, or the grant would name an action Cedar can't emit.
        assert_eq!(
            Permission::RunPipeline(PipelineId::new("_")).key(),
            super::RUN_PIPELINE_PERMISSION_KEY
        );
    }
}
