use crate::domain::entities::Job;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::JobStatus;
use crate::domain::value_objects::{JobId, PaginatedResult, PaginationParams, PipelineId};
use async_trait::async_trait;

/// Repository trait for Job entity
#[async_trait]
pub trait JobRepository: Send + Sync {
    /// Create a job
    async fn create(&self, job: &Job) -> DomainResult<Job>;

    /// Find a job by ID
    async fn find_by_id(&self, id: &JobId) -> DomainResult<Job>;

    /// Update a job
    async fn update(&self, job: &Job) -> DomainResult<Job>;

    /// Delete a job by ID
    async fn delete(&self, id: &JobId) -> DomainResult<()>;

    /// List all jobs
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>>;

    /// List jobs by status
    async fn list_by_status(
        &self,
        status: &JobStatus,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>>;

    /// List jobs by pipeline
    async fn list_by_pipeline(
        &self,
        pipeline_id: &PipelineId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>>;
}
