use crate::domain::entities::{OrganizationId, Pipeline, PipelineId, ProjectId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait PipelineRepository {
    async fn create(&self, pipeline: &Pipeline) -> DomainResult<Pipeline>;

    async fn find_by_id(&self, id: &PipelineId) -> DomainResult<Pipeline>;

    async fn update(&self, pipeline: &Pipeline) -> DomainResult<Pipeline>;

    async fn delete(&self, id: &PipelineId) -> DomainResult<()>;

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>>;

    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>>;

    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>>;
}
