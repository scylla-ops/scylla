use crate::application::JobLogRepository;
use crate::domain::entities::{JobId, JobLog, JobLogId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::pipeline::NodeId;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct JobLogUseCases<R: JobLogRepository> {
    repo: Arc<R>,
}

impl<R: JobLogRepository> JobLogUseCases<R> {
    #[instrument(skip(self, log))]
    pub async fn create(&self, log: &JobLog) -> DomainResult<JobLog> {
        self.repo.create(log).await
    }

    #[instrument(skip(self), fields(id = %id))]
    pub async fn get(&self, id: &JobLogId) -> DomainResult<JobLog> {
        self.repo.find_by_id(id).await
    }

    #[instrument(skip(self), fields(job_id = %job_id))]
    pub async fn list_by_job(
        &self,
        job_id: &JobId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>> {
        self.repo.list_by_job(job_id, pagination).await
    }

    #[instrument(skip(self), fields(job_id = %job_id, node_id = %node_id))]
    pub async fn list_by_job_and_node(
        &self,
        job_id: &JobId,
        node_id: &NodeId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>> {
        self.repo
            .list_by_job_and_node(job_id, node_id, pagination)
            .await
    }
}
