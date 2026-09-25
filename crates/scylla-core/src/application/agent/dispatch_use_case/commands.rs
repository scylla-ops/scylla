use super::{DispatchOutcome, DispatchUseCases};
use crate::application::actions::service_only;
use crate::application::agent::dispatch::{JobDispatch, assemble_dispatch};
use crate::domain::errors::DomainResult;
use crate::domain::job::Job;
use async_trait::async_trait;
use scylla_extension::{
    Access, Authorized, Command, Committed, Describe, Persist, Prepare, Prepared, Run,
};
use tracing::warn;

/// One pass over the jobs stored while no eligible agent was connected. A job whose dispatch
/// cannot be assembled is logged and skipped; `Committed` is the jobs an agent took.
#[derive(Debug)]
pub struct DispatchPendingJobs;

impl Describe for DispatchPendingJobs {
    fn access(&self) -> Access {
        Access::Authenticated
    }
}

impl Command for DispatchPendingJobs {
    type Staged = Vec<(Job, JobDispatch)>;
    type Committed = Vec<Job>;
}

#[async_trait]
impl Run<Prepare<DispatchPendingJobs>> for DispatchUseCases {
    async fn run(
        &self,
        input: Authorized<DispatchPendingJobs>,
    ) -> DomainResult<Prepared<DispatchPendingJobs>> {
        service_only(input.caller())?;
        let jobs = self.job_repo.list_pending_unassigned().await?;
        let mut staged = Vec::with_capacity(jobs.len());
        for job in jobs {
            match assemble_dispatch(&*self.pipeline_repo, &*self.secret_resolver, &job).await {
                Ok(dispatch) => staged.push((job, dispatch)),
                Err(e) => {
                    warn!(job_id = %job.id(), error = %e, "pending-job drain: dispatch assembly failed; skipping");
                }
            }
        }
        Ok(input.prepared(staged))
    }
}

#[async_trait]
impl Run<Persist<DispatchPendingJobs>> for DispatchUseCases {
    async fn run(
        &self,
        input: Prepared<DispatchPendingJobs>,
    ) -> DomainResult<Committed<DispatchPendingJobs>> {
        input
            .commit(async |staged| {
                let mut placed = Vec::new();
                for (mut job, dispatch) in staged {
                    if let DispatchOutcome::Dispatched(_) = self.place(&mut job, &dispatch).await {
                        placed.push(job);
                    }
                }
                Ok(placed)
            })
            .await
    }
}
