use crate::domain::entities::{AppId, Job, JobId, OrganizationId, PipelineId, ProjectId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait JobRepository {
    async fn create(&self, job: &Job) -> DomainResult<Job>;

    async fn find_by_id(&self, id: &JobId) -> DomainResult<Job>;

    async fn update(&self, job: &Job) -> DomainResult<Job>;

    /// Record which agent (app) executed a job. Targeted column update so it
    /// can't clobber concurrent status/node writes from the agent stream.
    async fn set_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()>;

    /// Jobs minted but never handed to an agent (status `pending`, no
    /// `agent_app_id`): the backlog to (re)dispatch when a worker connects.
    /// Oldest first (FIFO); not paginated — the scheduler drains the whole set.
    async fn list_pending_unassigned(&self) -> DomainResult<Vec<Job>>;

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
