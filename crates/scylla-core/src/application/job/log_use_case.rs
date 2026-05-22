use crate::application::caller::CallerContext;
use crate::application::{JobLogRepository, PermissionService};
use crate::domain::entities::{JobId, JobLog, JobLogId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::pipeline::NodeId;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct JobLogUseCases<R: JobLogRepository, PS: PermissionService> {
    repo: Arc<R>,
    permission_service: Arc<PS>,
}

impl<R: JobLogRepository, PS: PermissionService> JobLogUseCases<R, PS> {
    #[instrument(skip(self, caller, log))]
    pub async fn create(&self, caller: &CallerContext, log: &JobLog) -> DomainResult<JobLog> {
        // Recorder-only path today; routed through the trait so tightening
        // (per-service action allowlists) is contained to Cedar.
        self.permission_service
            .check(caller, Permission::WriteJobLogs(log.job_id().clone()))
            .await?;
        self.repo.create(log).await
    }

    /// Append a log line emitted by a agent over its stream. Gated by
    /// [`Permission::WriteJobLog`] — the action the agent role confers — so a
    /// agent can write logs without the broader `writeJobLogs` recorder grant.
    #[instrument(skip(self, caller, log))]
    pub async fn append(&self, caller: &CallerContext, log: &JobLog) -> DomainResult<JobLog> {
        self.permission_service
            .check(caller, Permission::WriteJobLog(log.job_id().clone()))
            .await?;
        self.repo.create(log).await
    }

    #[instrument(skip(self, caller), fields(id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &JobLogId) -> DomainResult<JobLog> {
        // Reading a single log line is gated by its job's read-logs grant; we
        // load first because the id alone does not carry the job context.
        let log = self.repo.find_by_id(id).await?;
        self.permission_service
            .check(caller, Permission::ReadJobLogs(log.job_id().clone()))
            .await?;
        Ok(log)
    }

    #[instrument(skip(self, caller), fields(job_id = %job_id))]
    pub async fn list_by_job(
        &self,
        caller: &CallerContext,
        job_id: &JobId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>> {
        self.permission_service
            .check(caller, Permission::ReadJobLogs(job_id.clone()))
            .await?;
        self.repo.list_by_job(job_id, pagination).await
    }

    #[instrument(skip(self, caller), fields(job_id = %job_id, node_id = %node_id))]
    pub async fn list_by_job_and_node(
        &self,
        caller: &CallerContext,
        job_id: &JobId,
        node_id: &NodeId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>> {
        self.permission_service
            .check(caller, Permission::ReadJobLogs(job_id.clone()))
            .await?;
        self.repo
            .list_by_job_and_node(job_id, node_id, pagination)
            .await
    }
}
