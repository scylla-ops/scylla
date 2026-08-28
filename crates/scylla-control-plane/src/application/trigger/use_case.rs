use crate::application::caller::CallerContext;
use crate::application::{
    AppRepository, CronSchedule, Grant, HashService, ORGANIZATION_TRIGGER_RUNNER_ROLE,
    PermissionService, PipelineRepository, PolicyControl, Principal, ProjectRepository, Scope,
    SecretCipher, TriggerRepository, next_fire_time,
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
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

/// Name of the per-organization machine App that pipeline runs fire as. One is
/// lazily provisioned per org on first trigger creation; it holds the
/// `organization-trigger-runner` role at org scope, which confers `runPipeline`
/// and nothing else (NOT the agent role, which only confers `executeJob`). It is
/// used in-process as `CallerContext::App`, never via a token, so its credential
/// is provisioned only to keep the App invariant.
pub(crate) const TRIGGER_RUNNER_APP_NAME: &str = "trigger-runner";
const RUNNER_SECRET_LABEL: &str = "default";

/// Manage a pipeline's triggers. Every method is Cedar-gated by
/// `ManageTriggers`; create/update additionally require `RunPipeline` on the
/// caller (anti-escalation: you cannot set up a trigger that runs a pipeline you
/// could not run yourself). Firing is handled separately (the firing engine /
/// FireTrigger), all converging on the unchanged `PipelineUseCases::run`.
// The derived `new` takes one arg per collaborator (8 with the cipher); that is
// the composition-root wiring, not a call-site ergonomics problem.
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
    /// AEAD cipher for the webhook signing secret at rest (same master key as
    /// project secrets). HMAC verification needs the plaintext back, so the secret
    /// is encrypted-reversible, never one-way hashed.
    cipher: Arc<dyn SecretCipher>,
    /// Computes a cron trigger's `next_fire_at` at create / update / re-enable —
    /// the same primitive the scheduler uses, so editing a schedule re-anchors it
    /// instead of leaving a stale due time.
    schedule: Arc<dyn CronSchedule>,
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
    /// Create a trigger. For a webhook source a fresh HMAC signing secret is
    /// generated, encrypted at rest, and returned ONCE as the second tuple element
    /// (the caller surfaces it to the user; it is never readable again). Cron
    /// triggers return `None`.
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

        let mut trigger = Trigger::create(pipeline_id, name, source, inputs)?;
        // Anchor the first cron occurrence (and validate the expression) up front,
        // through the same primitive the scheduler uses.
        self.schedule_next(&mut trigger)?;

        // Webhook triggers carry a generated signing secret (encrypted at rest,
        // plaintext returned once); cron triggers carry none.
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
        // Same anti-escalation fence as create: editing what a trigger fires
        // (source/inputs) requires the right to run the pipeline.
        self.permission_service
            .check(
                caller,
                Permission::RunPipeline(trigger.pipeline_id().clone()),
            )
            .await?;
        trigger.update(name, source, inputs)?;
        // Re-anchor next_fire_at from now: a changed cron expression takes effect
        // immediately (no stale fire at the old time); an unchanged one recomputes
        // to the same occurrence; a webhook clears it.
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
        // Re-enabling re-anchors from now (no catch-up fire at a stale past time);
        // disabling structurally drops the due time (see `Trigger::disable`).
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

    /// (Re)anchor `next_fire_at` through the shared [`next_fire_time`] primitive:
    /// the cron's next occurrence after now, or `None` for a webhook. The single
    /// place CRUD touches scheduling — same rule the scheduler applies.
    fn schedule_next(&self, trigger: &mut Trigger) -> DomainResult<()> {
        let next = next_fire_time(trigger, &*self.schedule, clock::now())?;
        trigger.set_next_fire_at(next);
        Ok(())
    }

    /// Ensure the org has its trigger-runner App (App + credential + an
    /// `organization-trigger-runner` grant at org scope), provisioned once and
    /// reused by every trigger in the org. Idempotent: a concurrent first-create that loses the
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
            // Lost the race to a concurrent first-create — the runner now exists.
            Err(DomainError::Conflict(_)) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// A fresh 256-bit webhook signing secret, hex-encoded (64 chars). High-entropy;
/// returned to the user once and stored only encrypted.
fn generate_webhook_secret() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
