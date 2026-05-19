use crate::domain::entities::{Job, JobId, OrganizationId, PipelineId, ProjectId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait JobRepository {
    async fn create(&self, job: &Job) -> DomainResult<Job>;

    async fn find_by_id(&self, id: &JobId) -> DomainResult<Job>;

    async fn update(&self, job: &Job) -> DomainResult<Job>;

    async fn delete(&self, id: &JobId) -> DomainResult<()>;

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>>;

    async fn list_by_pipeline(
        &self,
        pipeline_id: &PipelineId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>>;

    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>>;

    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>>;
}
