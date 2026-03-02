use derive_more::Constructor;
use domain::entities::{Job, JobId, OrganizationId, PipelineId, ProjectId};
use domain::errors::DomainResult;
use domain::ports::JobRepository;
use domain::value_objects::{PaginatedResult, PaginationParams};
use std::sync::Arc;

#[derive(Constructor)]
pub struct JobUseCases<J: JobRepository> {
    job_repo: Arc<J>,
}

impl<J: JobRepository> JobUseCases<J> {
    pub async fn get(&self, id: &JobId) -> DomainResult<Job> {
        self.job_repo.find_by_id(id).await
    }

    pub async fn delete(&self, id: &JobId) -> DomainResult<()> {
        self.job_repo.find_by_id(id).await?;
        self.job_repo.delete(id).await
    }

    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo.list_all(pagination).await
    }

    pub async fn list_by_pipeline(
        &self,
        pipeline_id: &PipelineId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo.list_by_pipeline(pipeline_id, pagination).await
    }

    pub async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo.list_by_project(project_id, pagination).await
    }

    pub async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo.list_by_organization(organization_id, pagination).await
    }
}
