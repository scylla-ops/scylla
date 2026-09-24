//! The job's writes. One block per command, in the order it runs: the struct, its permission,
//! its payload types, what `Prepare` builds, what `Persist` writes.

use super::JobUseCases;
use crate::application::job::JobEvent;
use crate::domain::errors::DomainResult;
use crate::domain::ids::JobId;
use crate::domain::job::{Job, NodeOutcome};
use crate::domain::permission::Permission;
use crate::domain::pipeline::NodeId;
use async_trait::async_trait;
use scylla_extension::{
    Authorized, Command, Committed, Deleted, Describe, Draft, Persist, Prepare, Prepared, Run,
};

/// One `WriteJobStatus` check; the reads and the write go through the port, so an agent needs
/// no `readJob`.
#[derive(Debug)]
pub struct RecordJobStatus {
    pub job_id: JobId,
    pub event: JobEvent,
}

impl Describe for RecordJobStatus {
    fn permission(&self) -> Permission {
        Permission::WriteJobStatus(self.job_id.clone())
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
    fn permission(&self) -> Permission {
        Permission::DeleteJob(self.id.clone())
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
