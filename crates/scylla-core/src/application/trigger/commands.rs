//! The trigger's writes. One block per command, in the order it runs: the struct, its
//! permission, its payload types, what `Prepare` builds, what `Persist` writes.

use super::TriggerUseCases;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, PipelineId, TriggerId};
use crate::domain::permission::Permission;
use crate::domain::trigger::{Trigger, TriggerInput, TriggerName, TriggerSource};
use async_trait::async_trait;
use scylla_extension::{
    Authorized, Command, Committed, Deleted, Describe, Draft, Persist, Prepare, Prepared, Run,
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
impl Run<Prepare<CreateTrigger>> for TriggerUseCases {
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
impl Run<Persist<CreateTrigger>> for TriggerUseCases {
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

#[derive(Debug)]
pub struct UpdateTrigger {
    pub id: TriggerId,
    pub name: TriggerName,
    pub source: TriggerSource,
    pub inputs: Vec<TriggerInput>,
}

impl Describe for UpdateTrigger {
    fn permission(&self) -> Permission {
        Permission::ManageTrigger(self.id.clone())
    }
}

impl Command for UpdateTrigger {
    type Staged = Draft<Trigger>;
    type Committed = Trigger;
}

#[async_trait]
impl Run<Prepare<UpdateTrigger>> for TriggerUseCases {
    // Anti-escalation, as for a create: it refuses, so it runs before anything is read.
    async fn run(&self, input: Authorized<UpdateTrigger>) -> DomainResult<Prepared<UpdateTrigger>> {
        let cmd = input.command();
        self.permission_service
            .check(
                input.caller(),
                Permission::RunTriggerPipeline(cmd.id.clone()),
            )
            .await?;
        let mut trigger = self.trigger_repo.find_by_id(&cmd.id).await?;
        trigger.update(cmd.name.clone(), cmd.source.clone(), cmd.inputs.clone())?;
        // Re-anchor from now so a changed expression takes effect at once.
        self.schedule_next(&mut trigger)?;
        Ok(input.prepared(Draft::new(trigger)))
    }
}

#[async_trait]
impl Run<Persist<UpdateTrigger>> for TriggerUseCases {
    async fn run(&self, input: Prepared<UpdateTrigger>) -> DomainResult<Committed<UpdateTrigger>> {
        input
            .commit(async |draft| self.trigger_repo.update(&draft.into_inner()).await)
            .await
    }
}

#[derive(Debug)]
pub struct SetTriggerEnabled {
    pub id: TriggerId,
    pub enabled: bool,
}

impl Describe for SetTriggerEnabled {
    fn permission(&self) -> Permission {
        Permission::ManageTrigger(self.id.clone())
    }
}

impl Command for SetTriggerEnabled {
    type Staged = Draft<Trigger>;
    type Committed = Trigger;
}

#[async_trait]
impl Run<Prepare<SetTriggerEnabled>> for TriggerUseCases {
    async fn run(
        &self,
        input: Authorized<SetTriggerEnabled>,
    ) -> DomainResult<Prepared<SetTriggerEnabled>> {
        let cmd = input.command();
        let mut trigger = self.trigger_repo.find_by_id(&cmd.id).await?;
        // Re-anchor from now: no catch-up fire at a stale past time.
        if cmd.enabled {
            trigger.enable();
            self.schedule_next(&mut trigger)?;
        } else {
            trigger.disable();
        }
        Ok(input.prepared(Draft::new(trigger)))
    }
}

#[async_trait]
impl Run<Persist<SetTriggerEnabled>> for TriggerUseCases {
    async fn run(
        &self,
        input: Prepared<SetTriggerEnabled>,
    ) -> DomainResult<Committed<SetTriggerEnabled>> {
        input
            .commit(async |draft| self.trigger_repo.update(&draft.into_inner()).await)
            .await
    }
}

#[derive(Debug)]
pub struct DeleteTrigger {
    pub id: TriggerId,
}

impl Describe for DeleteTrigger {
    fn permission(&self) -> Permission {
        Permission::ManageTrigger(self.id.clone())
    }
}

impl Command for DeleteTrigger {
    type Staged = Trigger;
    type Committed = Deleted<Trigger>;
}

#[async_trait]
impl Run<Prepare<DeleteTrigger>> for TriggerUseCases {
    async fn run(&self, input: Authorized<DeleteTrigger>) -> DomainResult<Prepared<DeleteTrigger>> {
        let trigger = self.trigger_repo.find_by_id(&input.command().id).await?;
        Ok(input.prepared(trigger))
    }
}

#[async_trait]
impl Run<Persist<DeleteTrigger>> for TriggerUseCases {
    async fn run(&self, input: Prepared<DeleteTrigger>) -> DomainResult<Committed<DeleteTrigger>> {
        input
            .commit(async |trigger| {
                self.trigger_repo.delete(trigger.id()).await?;
                Ok(Deleted::new(trigger))
            })
            .await
    }
}

fn generate_webhook_secret() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
