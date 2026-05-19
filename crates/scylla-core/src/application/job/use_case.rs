use crate::application::JobRepository;
use crate::domain::entities::{Job, JobId, OrganizationId, PipelineId, ProjectId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct JobUseCases<J: JobRepository> {
    job_repo: Arc<J>,
}

impl<J: JobRepository> JobUseCases<J> {
    #[instrument(skip(self, job))]
    pub async fn create(&self, job: &Job) -> DomainResult<Job> {
        self.job_repo.create(job).await
    }

    #[instrument(skip(self), fields(job_id = %id))]
    pub async fn get(&self, id: &JobId) -> DomainResult<Job> {
        self.job_repo.find_by_id(id).await
    }

    #[instrument(skip(self, job))]
    pub async fn update(&self, job: &Job) -> DomainResult<Job> {
        self.job_repo.update(job).await
    }

    #[instrument(skip(self), fields(job_id = %id))]
    pub async fn delete(&self, id: &JobId) -> DomainResult<()> {
        self.job_repo.find_by_id(id).await?;
        self.job_repo.delete(id).await
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo.list_all(pagination).await
    }

    #[instrument(skip(self), fields(pipeline_id = %pipeline_id))]
    pub async fn list_by_pipeline(
        &self,
        pipeline_id: &PipelineId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo
            .list_by_pipeline(pipeline_id, pagination)
            .await
    }

    #[instrument(skip(self), fields(project_id = %project_id))]
    pub async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo.list_by_project(project_id, pagination).await
    }

    #[instrument(skip(self), fields(org_id = %organization_id))]
    pub async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo
            .list_by_organization(organization_id, pagination)
            .await
    }
}
