//! The job log's writes. One block per command, in the order it runs: the struct, its
//! permission, its payload types, what `Prepare` builds, what `Persist` writes.

use super::JobLogUseCases;
use crate::domain::errors::DomainResult;
use crate::domain::job::JobLog;
use crate::domain::permission::Permission;
use async_trait::async_trait;
use scylla_extension::{
    Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};

#[derive(Debug)]
pub struct AppendJobLog {
    pub log: JobLog,
}

impl Describe for AppendJobLog {
    fn permission(&self) -> Permission {
        Permission::AppendJobLog(self.log.job_id().clone())
    }
}

impl Command for AppendJobLog {
    type Staged = Draft<JobLog>;
    type Committed = JobLog;
}

#[async_trait]
impl Run<Prepare<AppendJobLog>> for JobLogUseCases {
    async fn run(&self, input: Authorized<AppendJobLog>) -> DomainResult<Prepared<AppendJobLog>> {
        let log = input.command().log.clone();
        Ok(input.prepared(Draft::new(log)))
    }
}

#[async_trait]
impl Run<Persist<AppendJobLog>> for JobLogUseCases {
    async fn run(&self, input: Prepared<AppendJobLog>) -> DomainResult<Committed<AppendJobLog>> {
        input
            .commit(async |draft| self.log_repo.create(&draft.into_inner()).await)
            .await
    }
}
