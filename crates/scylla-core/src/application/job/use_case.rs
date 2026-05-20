use crate::application::caller::CallerContext;
use crate::application::{JobRepository, PermissionService};
use crate::domain::entities::{Job, JobId, OrganizationId, PipelineId, ProjectId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct JobUseCases<J: JobRepository, PS: PermissionService> {
    job_repo: Arc<J>,
    permission_service: Arc<PS>,
}

impl<J: JobRepository, PS: PermissionService> JobUseCases<J, PS> {
    /// Persist a freshly minted job. Called by orchestrators (e.g. pipeline
    /// run) and by the recorder when reconciling broker events; in both cases
    /// the caller is `Service` and the Cedar service-permit rule allows it.
    #[instrument(skip(self, caller, job))]
    pub async fn create(&self, caller: &CallerContext, job: &Job) -> DomainResult<Job> {
        // `create` is service-only today (orchestrator / recorder); the Cedar
        // service-permit rule admits it. End-user job creation has no policy yet.
        self.permission_service
            .check(caller, Permission::CreateJob)
            .await?;
        self.job_repo.create(job).await
    }

    #[instrument(skip(self, caller), fields(job_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &JobId) -> DomainResult<Job> {
        self.permission_service
            .check(caller, Permission::ReadJob(id.clone()))
            .await?;
        self.job_repo.find_by_id(id).await
    }

    #[instrument(skip(self, caller, job))]
    pub async fn update(&self, caller: &CallerContext, job: &Job) -> DomainResult<Job> {
        self.permission_service
            .check(caller, Permission::WriteJob(job.id().clone()))
            .await?;
        self.job_repo.update(job).await
    }

    #[instrument(skip(self, caller), fields(job_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &JobId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeleteJob(id.clone()))
            .await?;
        self.job_repo.find_by_id(id).await?;
        self.job_repo.delete(id).await
    }

    #[instrument(skip(self, caller))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.permission_service
            .check(caller, Permission::ListJobs)
            .await?;
        self.job_repo.list_all(pagination).await
    }

    #[instrument(skip(self, caller), fields(pipeline_id = %pipeline_id))]
    pub async fn list_by_pipeline(
        &self,
        caller: &CallerContext,
        pipeline_id: &PipelineId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.permission_service
            .check(caller, Permission::ListJobsByPipeline(pipeline_id.clone()))
            .await?;
        self.job_repo
            .list_by_pipeline(pipeline_id, pagination)
            .await
    }

    #[instrument(skip(self, caller), fields(project_id = %project_id))]
    pub async fn list_by_project(
        &self,
        caller: &CallerContext,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.permission_service
            .check(caller, Permission::ListJobsByProject(project_id.clone()))
            .await?;
        self.job_repo.list_by_project(project_id, pagination).await
    }

    #[instrument(skip(self, caller), fields(org_id = %organization_id))]
    pub async fn list_by_organization(
        &self,
        caller: &CallerContext,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.permission_service
            .check(
                caller,
                Permission::ListJobsByOrganization(organization_id.clone()),
            )
            .await?;
        self.job_repo
            .list_by_organization(organization_id, pagination)
            .await
    }
}
