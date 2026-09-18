use crate::application::JobLogRepository;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{JobId, JobLogId};
use crate::domain::job::JobLog;
use crate::domain::permission::Permission;
use crate::domain::pipeline::NodeId;
use derive_more::Constructor;
use scylla_auth::authz::PermissionService;
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
        self.permission_service
            .check(caller, Permission::WriteJobLogs(log.job_id().clone()))
            .await?;
        self.repo.create(log).await
    }

    #[instrument(skip(self, caller, log))]
    pub async fn append(&self, caller: &CallerContext, log: &JobLog) -> DomainResult<JobLog> {
        self.permission_service
            .check(caller, Permission::AppendJobLog(log.job_id().clone()))
            .await?;
        self.repo.create(log).await
    }

    #[instrument(skip_all, fields(id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &JobLogId) -> DomainResult<JobLog> {
        let log = self.repo.find_by_id(id).await?;
        self.permission_service
            .check(caller, Permission::ReadJobLogs(log.job_id().clone()))
            .await?;
        Ok(log)
    }

    #[instrument(skip_all, fields(job_id = %job_id))]
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

    #[instrument(skip_all, fields(job_id = %job_id, node_id = %node_id))]
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
