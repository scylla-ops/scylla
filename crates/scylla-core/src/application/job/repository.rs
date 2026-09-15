use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, JobId, OrganizationId, PipelineId, ProjectId};
use crate::domain::job::Job;
use async_trait::async_trait;

#[async_trait]
pub trait JobRepository {
    async fn create(&self, job: &Job) -> DomainResult<Job>;

    async fn find_by_id(&self, id: &JobId) -> DomainResult<Job>;

    async fn update(&self, job: &Job) -> DomainResult<Job>;

    /// Targeted column update: must not clobber concurrent status writes from the agent stream.
    async fn set_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()>;

    async fn list_pending_unassigned(&self) -> DomainResult<Vec<Job>>;

    async fn orphan_running_without_agents(&self, connected: &[AppId]) -> DomainResult<u64>;

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
