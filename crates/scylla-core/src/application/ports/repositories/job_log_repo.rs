use crate::domain::entities::{JobId, JobLog, JobLogId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::pipeline::NodeId;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait JobLogRepository {
    async fn create(&self, log: &JobLog) -> DomainResult<JobLog>;

    async fn find_by_id(&self, id: &JobLogId) -> DomainResult<JobLog>;

    async fn list_by_job(
        &self,
        job_id: &JobId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>>;

    async fn list_by_job_and_node(
        &self,
        job_id: &JobId,
        node_id: &NodeId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>>;

    /// Return every persisted log for `job_id` (optionally filtered by `node_id`),
    /// ordered by timestamp ascending. Used as the historical snapshot for tailing.
    async fn list_all_by_job(
        &self,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<Vec<JobLog>>;
}
