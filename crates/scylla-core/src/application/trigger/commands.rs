//! The trigger's writes. One block per command, in the order it runs: the struct, its
//! access, its payload types, what `Prepare` builds, what `Persist` writes.

use super::{TriggerUseCases, next_fire_time};
use crate::application::actions::service_only;
use crate::domain::clock;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{OrganizationId, PipelineId, TriggerId};
use crate::domain::permission::Permission;
use crate::domain::trigger::{Trigger, TriggerInput, TriggerName, TriggerSource};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla_extension::{
    Access, Authorized, Command, Committed, Deleted, Describe, Draft, Persist, Prepare, Prepared,
    Run,
};
use tracing::warn;
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

/// No `Debug`: `webhook_secret` is the plaintext the caller sees once.
pub struct CreatedTrigger {
    pub trigger: Trigger,
    pub webhook_secret: Option<String>,
}

// Managing triggers must not give run rights, so a create and an update also ask for them.
impl Describe for CreateTrigger {
    fn access(&self) -> Access {
        Access::RequiresAll(vec![
            Permission::ManageTriggers(self.pipeline_id.clone()),
            Permission::RunPipeline(self.pipeline_id.clone()),
        ])
    }
}

impl Command for CreateTrigger {
    type Staged = Draft<NewTrigger>;
    type Committed = CreatedTrigger;
}

#[async_trait]
impl Run<Prepare<CreateTrigger>> for TriggerUseCases {
    async fn run(&self, input: Authorized<CreateTrigger>) -> DomainResult<Prepared<CreateTrigger>> {
        let cmd = input.command();
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
                Ok(CreatedTrigger {
                    trigger: stored,
                    webhook_secret,
                })
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
    fn access(&self) -> Access {
        Access::RequiresAll(vec![
            Permission::ManageTrigger(self.id.clone()),
            Permission::RunTriggerPipeline(self.id.clone()),
        ])
    }
}

impl Command for UpdateTrigger {
    type Staged = Draft<Trigger>;
    type Committed = Trigger;
}

#[async_trait]
impl Run<Prepare<UpdateTrigger>> for TriggerUseCases {
    async fn run(&self, input: Authorized<UpdateTrigger>) -> DomainResult<Prepared<UpdateTrigger>> {
        let cmd = input.command();
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
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageTrigger(self.id.clone()))
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
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageTrigger(self.id.clone()))
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

/// The outcome of a fire, sent by the trigger firer as a service. The row that the fire loaded
/// is written back with its observation, as before.
#[derive(Debug)]
pub struct RecordTriggerFire {
    pub trigger: Trigger,
    pub status: &'static str,
}

impl Describe for RecordTriggerFire {
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageTrigger(self.trigger.id().clone()))
    }
}

impl Command for RecordTriggerFire {
    type Staged = Draft<Trigger>;
    type Committed = Trigger;
}

#[async_trait]
impl Run<Prepare<RecordTriggerFire>> for TriggerUseCases {
    async fn run(
        &self,
        input: Authorized<RecordTriggerFire>,
    ) -> DomainResult<Prepared<RecordTriggerFire>> {
        let cmd = input.command();
        let mut trigger = cmd.trigger.clone();
        trigger.mark_fired(clock::now(), cmd.status);
        Ok(input.prepared(Draft::new(trigger)))
    }
}

#[async_trait]
impl Run<Persist<RecordTriggerFire>> for TriggerUseCases {
    async fn run(
        &self,
        input: Prepared<RecordTriggerFire>,
    ) -> DomainResult<Committed<RecordTriggerFire>> {
        input
            .commit(async |draft| self.trigger_repo.update(&draft.into_inner()).await)
            .await
    }
}

/// One pass over the cron triggers that have no next fire time. An invalid expression and a
/// failed write are logged, and that trigger stays unscheduled.
#[derive(Debug)]
pub struct ScheduleCronTriggers;

impl Describe for ScheduleCronTriggers {
    fn access(&self) -> Access {
        Access::Authenticated
    }
}

impl Command for ScheduleCronTriggers {
    type Staged = Draft<Vec<Trigger>>;
    type Committed = Vec<Trigger>;
}

#[async_trait]
impl Run<Prepare<ScheduleCronTriggers>> for TriggerUseCases {
    async fn run(
        &self,
        input: Authorized<ScheduleCronTriggers>,
    ) -> DomainResult<Prepared<ScheduleCronTriggers>> {
        service_only(input.caller())?;
        let mut staged = Vec::new();
        for mut trigger in self.trigger_repo.list_unscheduled_cron().await? {
            match next_fire_time(&trigger, &*self.schedule, clock::now()) {
                Ok(Some(next)) => {
                    trigger.set_next_fire_at(Some(next));
                    staged.push(trigger);
                }
                Ok(None) => {}
                Err(e) => {
                    warn!(trigger_id = %trigger.id(), error = %e, "cron seed: invalid expression; trigger will not fire");
                }
            }
        }
        Ok(input.prepared(Draft::new(staged)))
    }
}

#[async_trait]
impl Run<Persist<ScheduleCronTriggers>> for TriggerUseCases {
    async fn run(
        &self,
        input: Prepared<ScheduleCronTriggers>,
    ) -> DomainResult<Committed<ScheduleCronTriggers>> {
        input
            .commit(async |draft| {
                let staged = draft.into_inner();
                let mut scheduled = Vec::with_capacity(staged.len());
                for trigger in staged {
                    match self.trigger_repo.update(&trigger).await {
                        Ok(trigger) => scheduled.push(trigger),
                        Err(e) => {
                            warn!(trigger_id = %trigger.id(), error = %e, "cron seed: could not persist next_fire_at");
                        }
                    }
                }
                Ok(scheduled)
            })
            .await
    }
}

/// One claim of the cron triggers due at `now`: the store advances each claimed row to its
/// next fire time in the same transaction, so no pass and no instance claims a row twice.
#[derive(Debug)]
pub struct ClaimDueTriggers {
    pub now: DateTime<Utc>,
    pub limit: i64,
}

impl Describe for ClaimDueTriggers {
    fn access(&self) -> Access {
        Access::Authenticated
    }
}

impl Command for ClaimDueTriggers {
    type Staged = (DateTime<Utc>, i64);
    type Committed = Vec<Trigger>;
}

#[async_trait]
impl Run<Prepare<ClaimDueTriggers>> for TriggerUseCases {
    async fn run(
        &self,
        input: Authorized<ClaimDueTriggers>,
    ) -> DomainResult<Prepared<ClaimDueTriggers>> {
        service_only(input.caller())?;
        let staged = (input.command().now, input.command().limit);
        Ok(input.prepared(staged))
    }
}

#[async_trait]
impl Run<Persist<ClaimDueTriggers>> for TriggerUseCases {
    async fn run(
        &self,
        input: Prepared<ClaimDueTriggers>,
    ) -> DomainResult<Committed<ClaimDueTriggers>> {
        input
            .commit(async |(now, limit)| {
                let compute_next = |trigger: &Trigger| -> DomainResult<DateTime<Utc>> {
                    next_fire_time(trigger, &*self.schedule, now)?.ok_or_else(|| {
                        DomainError::internal("non-cron trigger reached the cron claim")
                    })
                };
                self.trigger_repo
                    .claim_due_cron(now, limit, &compute_next)
                    .await
            })
            .await
    }
}

fn generate_webhook_secret() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
