//! The job's writes. One block per command, in the order it runs: the struct, its access,
//! its payload types, what `Prepare` builds, what `Persist` writes.

use super::JobUseCases;
use crate::application::actions::service_only;
use crate::application::job::JobEvent;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, JobId};
use crate::domain::job::{Job, NodeOutcome};
use crate::domain::permission::Permission;
use crate::domain::pipeline::NodeId;
use async_trait::async_trait;
use scylla_extension::{
    Access, Authorized, Command, Committed, Deleted, Describe, Draft, Persist, Prepare, Prepared,
    Run,
};

/// One `WriteJobStatus` check; the reads and the write go through the port, so an agent needs
/// no `readJob`.
#[derive(Debug)]
pub struct RecordJobStatus {
    pub job_id: JobId,
    pub event: JobEvent,
}

impl Describe for RecordJobStatus {
    fn access(&self) -> Access {
        Access::Requires(Permission::WriteJobStatus(self.job_id.clone()))
    }
}

impl Command for RecordJobStatus {
    type Staged = Draft<Job>;
    type Committed = Job;
}

#[async_trait]
impl Run<Prepare<RecordJobStatus>> for JobUseCases {
    async fn run(
        &self,
        input: Authorized<RecordJobStatus>,
    ) -> DomainResult<Prepared<RecordJobStatus>> {
        let cmd = input.command();
        let job = self.job_repo.find_by_id(&cmd.job_id).await?;
        let now = chrono::Utc::now();
        let job = match &cmd.event {
            JobEvent::JobStarted => job.start()?,
            JobEvent::NodeStarted { node_id } => {
                job.apply_node_started(&NodeId::new(node_id)?, now)?
            }
            JobEvent::NodeCompleted { node_id } => {
                job.apply_node_finished(&NodeId::new(node_id)?, NodeOutcome::Completed, now)?
            }
            JobEvent::NodeFailed { node_id, .. } => {
                job.apply_node_finished(&NodeId::new(node_id)?, NodeOutcome::Failed, now)?
            }
            JobEvent::NodeSkipped { node_id } => {
                job.apply_node_skipped(&NodeId::new(node_id)?, now)?
            }
            JobEvent::JobCompleted => job.complete()?,
            JobEvent::JobFailed { .. } => job.fail()?,
        };
        Ok(input.prepared(Draft::new(job)))
    }
}

#[async_trait]
impl Run<Persist<RecordJobStatus>> for JobUseCases {
    async fn run(
        &self,
        input: Prepared<RecordJobStatus>,
    ) -> DomainResult<Committed<RecordJobStatus>> {
        input
            .commit(async |draft| self.job_repo.update(&draft.into_inner()).await)
            .await
    }
}

#[derive(Debug)]
pub struct DeleteJob {
    pub id: JobId,
}

impl Describe for DeleteJob {
    fn access(&self) -> Access {
        Access::Requires(Permission::DeleteJob(self.id.clone()))
    }
}

impl Command for DeleteJob {
    type Staged = Job;
    type Committed = Deleted<Job>;
}

#[async_trait]
impl Run<Prepare<DeleteJob>> for JobUseCases {
    async fn run(&self, input: Authorized<DeleteJob>) -> DomainResult<Prepared<DeleteJob>> {
        let job = self.job_repo.find_by_id(&input.command().id).await?;
        Ok(input.prepared(job))
    }
}

#[async_trait]
impl Run<Persist<DeleteJob>> for JobUseCases {
    async fn run(&self, input: Prepared<DeleteJob>) -> DomainResult<Committed<DeleteJob>> {
        input
            .commit(async |job| {
                self.job_repo.delete(job.id()).await?;
                Ok(Deleted::new(job))
            })
            .await
    }
}

/// One reconciliation pass: every running job whose agent is not in `connected` becomes
/// orphaned. `Committed` is the count.
#[derive(Debug)]
pub struct ReapOrphanedJobs {
    pub connected: Vec<AppId>,
}

impl Describe for ReapOrphanedJobs {
    fn access(&self) -> Access {
        Access::Authenticated
    }
}

impl Command for ReapOrphanedJobs {
    type Staged = Vec<AppId>;
    type Committed = u64;
}

#[async_trait]
impl Run<Prepare<ReapOrphanedJobs>> for JobUseCases {
    async fn run(
        &self,
        input: Authorized<ReapOrphanedJobs>,
    ) -> DomainResult<Prepared<ReapOrphanedJobs>> {
        service_only(input.caller())?;
        let connected = input.command().connected.clone();
        Ok(input.prepared(connected))
    }
}

#[async_trait]
impl Run<Persist<ReapOrphanedJobs>> for JobUseCases {
    async fn run(
        &self,
        input: Prepared<ReapOrphanedJobs>,
    ) -> DomainResult<Committed<ReapOrphanedJobs>> {
        input
            .commit(async |connected| {
                self.job_repo
                    .orphan_running_without_agents(&connected)
                    .await
            })
            .await
    }
}
