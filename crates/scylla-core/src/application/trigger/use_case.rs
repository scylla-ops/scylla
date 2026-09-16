use crate::application::{
    AppRepository, CronSchedule, HashService, PipelineRepository, ProjectRepository, SecretCipher,
    TriggerRepository, next_fire_time, quota,
};
use crate::domain::app::{App, AppCredential};
use crate::domain::app::{AppName, AppSecretLabel};
use crate::domain::clock;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{OrganizationId, PipelineId, TriggerId};
use crate::domain::permission::Permission;
use crate::domain::role::RoleName;
use crate::domain::trigger::Trigger;
use crate::domain::trigger::{TriggerInput, TriggerName, TriggerSource};
use derive_more::Constructor;
use scylla_auth::authz::{
    Grant, ORGANIZATION_TRIGGER_RUNNER_ROLE, PermissionService, PolicyControl, Principal, Scope,
};
use scylla_auth::caller::CallerContext;
use scylla_extension::{QuotaPolicy, Resource};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

/// Holds `organization-trigger-runner` (only `runPipeline`); used in-process, never via a token.
pub(crate) const TRIGGER_RUNNER_APP_NAME: &str = "trigger-runner";
const RUNNER_SECRET_LABEL: &str = "default";

#[allow(clippy::too_many_arguments)]
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
    /// Reversible: HMAC verification needs the plaintext back.
    cipher: Arc<dyn SecretCipher>,
    schedule: Arc<dyn CronSchedule>,
    quota: Arc<dyn QuotaPolicy>,
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
    #[instrument(skip_all, fields(pipeline_id = %pipeline_id, name = %name))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        pipeline_id: PipelineId,
        name: TriggerName,
        source: TriggerSource,
        inputs: Vec<TriggerInput>,
    ) -> DomainResult<(Trigger, Option<String>)> {
        self.permission_service
            .check(caller, Permission::ManageTriggers(pipeline_id.clone()))
            .await?;
        // Anti-escalation: managing triggers must not launder run rights.
        self.permission_service
            .check(caller, Permission::RunPipeline(pipeline_id.clone()))
            .await?;

        let pipeline = self.pipeline_repo.find_by_id(&pipeline_id).await?;
        // Before the runner App is provisioned: a denied create must leave nothing behind.
        quota::enforce(
            self.quota
                .check(Resource::Trigger, pipeline_id.as_str())
                .await,
        )?;
        let project = self.project_repo.find_by_id(pipeline.project_id()).await?;
        self.ensure_runner_app(project.organization_id()).await?;

        let mut trigger = Trigger::create(pipeline_id, name, source, inputs)?;
        self.schedule_next(&mut trigger)?;

        let (secret_plaintext, secret_enc) = match trigger.source() {
            TriggerSource::Webhook(_) => {
                let plaintext = generate_webhook_secret();
                let enc = self.cipher.encrypt(&plaintext)?;
                (Some(plaintext), Some(enc))
            }
            TriggerSource::Cron(_) => (None, None),
        };

        let stored = self
            .trigger_repo
            .create(&trigger, secret_enc.as_deref())
            .await?;
        Ok((stored, secret_plaintext))
    }

    #[instrument(skip_all, fields(trigger_id = %trigger_id))]
    pub async fn get(
        &self,
        caller: &CallerContext,
        trigger_id: &TriggerId,
    ) -> DomainResult<Trigger> {
        let trigger = self.trigger_repo.find_by_id(trigger_id).await?;
        self.permission_service
            .check(
                caller,
                Permission::ManageTriggers(trigger.pipeline_id().clone()),
            )
            .await?;
        Ok(trigger)
    }

    #[instrument(skip_all, fields(pipeline_id = %pipeline_id))]
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

    #[instrument(skip_all, fields(trigger_id = %trigger_id))]
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
            .check(
                caller,
                Permission::ManageTriggers(trigger.pipeline_id().clone()),
            )
            .await?;
        self.permission_service
            .check(
                caller,
                Permission::RunPipeline(trigger.pipeline_id().clone()),
            )
            .await?;
        trigger.update(name, source, inputs)?;
        // Re-anchor from now so a changed expression takes effect at once.
        self.schedule_next(&mut trigger)?;
        self.trigger_repo.update(&trigger).await
    }

    #[instrument(skip_all, fields(trigger_id = %trigger_id, enabled))]
    pub async fn set_enabled(
        &self,
        caller: &CallerContext,
        trigger_id: &TriggerId,
        enabled: bool,
    ) -> DomainResult<Trigger> {
        let mut trigger = self.trigger_repo.find_by_id(trigger_id).await?;
        self.permission_service
            .check(
                caller,
                Permission::ManageTriggers(trigger.pipeline_id().clone()),
            )
            .await?;
        // Re-anchor from now: no catch-up fire at a stale past time.
        if enabled {
            trigger.enable();
            self.schedule_next(&mut trigger)?;
        } else {
            trigger.disable();
        }
        self.trigger_repo.update(&trigger).await
    }

    #[instrument(skip_all, fields(trigger_id = %trigger_id))]
    pub async fn delete(&self, caller: &CallerContext, trigger_id: &TriggerId) -> DomainResult<()> {
        let trigger = self.trigger_repo.find_by_id(trigger_id).await?;
        self.permission_service
            .check(
                caller,
                Permission::ManageTriggers(trigger.pipeline_id().clone()),
            )
            .await?;
        self.trigger_repo.delete(trigger_id).await
    }

    fn schedule_next(&self, trigger: &mut Trigger) -> DomainResult<()> {
        let next = next_fire_time(trigger, &*self.schedule, clock::now())?;
        trigger.set_next_fire_at(next);
        Ok(())
    }

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

fn generate_webhook_secret() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
