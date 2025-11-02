use crate::domain::entities::Pipeline;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams, PipelineId};
use async_trait::async_trait;

/// Repository trait for Pipeline entity
#[async_trait]
pub trait PipelineRepository: Send + Sync {
    /// Save a pipeline
    async fn create(&self, pipeline: &Pipeline) -> DomainResult<Pipeline>;

    /// Find a pipeline by ID
    async fn find_by_id(&self, id: &PipelineId) -> DomainResult<Pipeline>;

    /// Update a pipeline
    async fn update(&self, pipeline: &Pipeline) -> DomainResult<Pipeline>;

    /// Delete a pipeline by ID
    async fn delete(&self, id: &PipelineId) -> DomainResult<()>;

    /// List all pipelines
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>>;
}
