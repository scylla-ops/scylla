//! The trigger's writes. One block per command, in the order it runs: the struct, its
//! permission, its payload types, what `Prepare` builds, what `Persist` writes.

use super::TriggerUseCases;
use crate::application::{
    AppRepository, HashService, PipelineRepository, ProjectRepository, TriggerRepository,
};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, PipelineId};
use crate::domain::permission::Permission;
use crate::domain::trigger::{Trigger, TriggerInput, TriggerName, TriggerSource};
use async_trait::async_trait;
use scylla_auth::authz::{PermissionService, PolicyControl};
use scylla_extension::{
    Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct CreateTrigger {
    pub pipeline_id: PipelineId,
    pub name: TriggerName,
    pub source: TriggerSource,
    pub inputs: Vec<TriggerInput>,
}

/// No `Debug`: `webhook_secret` is the plaintext the caller sees once. The runner app of the
/// organization is provisioned with the trigger, so its organization is staged too.
pub struct NewTrigger {
    pub trigger: Trigger,
    pub organization_id: OrganizationId,
    pub webhook_secret: Option<String>,
    pub webhook_secret_enc: Option<Vec<u8>>,
}

impl Describe for CreateTrigger {
    fn permission(&self) -> Permission {
        Permission::ManageTriggers(self.pipeline_id.clone())
    }
}

impl Command for CreateTrigger {
    type Staged = Draft<NewTrigger>;
    type Committed = (Trigger, Option<String>);
}

#[async_trait]
impl<T, P, PR, A, H, PC, PS> Run<Prepare<CreateTrigger>> for TriggerUseCases<T, P, PR, A, H, PC, PS>
where
    T: TriggerRepository + Send + Sync,
    P: PipelineRepository + Send + Sync,
    PR: ProjectRepository + Send + Sync,
    A: AppRepository,
    H: HashService + Send + Sync,
    PC: PolicyControl,
    PS: PermissionService,
{
    // Anti-escalation: managing triggers must not launder run rights. It refuses, so it runs
    // before anything is read.
    async fn run(&self, input: Authorized<CreateTrigger>) -> DomainResult<Prepared<CreateTrigger>> {
        let cmd = input.command();
        self.permission_service
            .check(
                input.caller(),
                Permission::RunPipeline(cmd.pipeline_id.clone()),
            )
            .await?;
        let pipeline = self.pipeline_repo.find_by_id(&cmd.pipeline_id).await?;
        let project = self.project_repo.find_by_id(pipeline.project_id()).await?;

        let mut trigger = Trigger::create(
            cmd.pipeline_id.clone(),
            cmd.name.clone(),
            cmd.source.clone(),
            cmd.inputs.clone(),
        )?;
        self.schedule_next(&mut trigger)?;

        let (webhook_secret, webhook_secret_enc) = match trigger.source() {
            TriggerSource::Webhook(_) => {
                let plaintext = generate_webhook_secret();
                let enc = self.cipher.encrypt(&plaintext)?;
                (Some(plaintext), Some(enc))
            }
            TriggerSource::Cron(_) => (None, None),
        };
        Ok(input.prepared(Draft::new(NewTrigger {
            trigger,
            organization_id: project.organization_id().clone(),
            webhook_secret,
            webhook_secret_enc,
        })))
    }
}

#[async_trait]
impl<T, P, PR, A, H, PC, PS> Run<Persist<CreateTrigger>> for TriggerUseCases<T, P, PR, A, H, PC, PS>
where
    T: TriggerRepository + Send + Sync,
    P: PipelineRepository + Send + Sync,
    PR: ProjectRepository + Send + Sync,
    A: AppRepository,
    H: HashService + Send + Sync,
    PC: PolicyControl,
    PS: PermissionService,
{
    async fn run(&self, input: Prepared<CreateTrigger>) -> DomainResult<Committed<CreateTrigger>> {
        input
            .commit(async |draft| {
                let NewTrigger {
                    trigger,
                    organization_id,
                    webhook_secret,
                    webhook_secret_enc,
                } = draft.into_inner();
                self.ensure_runner_app(&organization_id).await?;
                let stored = self
                    .trigger_repo
                    .create(&trigger, webhook_secret_enc.as_deref())
                    .await?;
                Ok((stored, webhook_secret))
            })
            .await
    }
}

fn generate_webhook_secret() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
