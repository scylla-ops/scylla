use crate::application::caller::CallerContext;
use crate::application::{
    JobDispatch, JobRepository, PermissionService, PipelineRepository, ProjectRepository,
};
use crate::domain::entities::{
    AppId, Job, JobId, OrganizationId, Pipeline, PipelineId, PipelineNode, ProjectId,
};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::pipeline::PipelineName;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct PipelineUseCases<
    P: PipelineRepository,
    PR: ProjectRepository,
    J: JobRepository,
    PS: PermissionService,
> {
    pipeline_repo: Arc<P>,
    project_repo: Arc<PR>,
    job_repo: Arc<J>,
    permission_service: Arc<PS>,
}

impl<P: PipelineRepository, PR: ProjectRepository, J: JobRepository, PS: PermissionService>
    PipelineUseCases<P, PR, J, PS>
{
    #[instrument(skip(self, caller, nodes), fields(name = %name, project_id = %project_id))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        name: PipelineName,
        project_id: ProjectId,
        nodes: Vec<PipelineNode>,
    ) -> DomainResult<Pipeline> {
        self.permission_service
            .check(caller, Permission::CreatePipeline(project_id.clone()))
            .await?;
        self.project_repo.find_by_id(&project_id).await?;
        let pipeline = Pipeline::create(name, project_id, nodes)?;
        self.pipeline_repo.create(&pipeline).await
    }

    #[instrument(skip(self, caller), fields(pipeline_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &PipelineId) -> DomainResult<Pipeline> {
        self.permission_service
            .check(caller, Permission::ReadPipeline(id.clone()))
            .await?;
        self.pipeline_repo.find_by_id(id).await
    }

    // Note: previously had `find_internal` for orchestration paths to bypass the
    // get-check; replaced by the consolidated `run()` method below which does a
    // single `Permission::RunPipeline` check and reads the pipeline via the repo.

    #[instrument(skip(self, caller, nodes), fields(pipeline_id = %id))]
    pub async fn update(
        &self,
        caller: &CallerContext,
        id: &PipelineId,
        name: Option<PipelineName>,
        nodes: Option<Vec<PipelineNode>>,
    ) -> DomainResult<Pipeline> {
        self.permission_service
            .check(caller, Permission::UpdatePipeline(id.clone()))
            .await?;

        let mut pipeline = self.pipeline_repo.find_by_id(id).await?;

        if let Some(new_name) = name {
            pipeline.update_name(new_name)?;
        }
        if let Some(new_nodes) = nodes {
            pipeline.update_nodes(new_nodes)?;
        }

        self.pipeline_repo.update(&pipeline).await
    }

    #[instrument(skip(self, caller), fields(pipeline_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &PipelineId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeletePipeline(id.clone()))
            .await?;
        self.pipeline_repo.find_by_id(id).await?;
        self.pipeline_repo.delete(id).await
    }

    #[instrument(skip(self, caller))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.permission_service
            .check(caller, Permission::ListPipelines)
            .await?;
        self.pipeline_repo.list_all(pagination).await
    }

    #[instrument(skip(self, caller), fields(project_id = %project_id))]
    pub async fn list_by_project(
        &self,
        caller: &CallerContext,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.permission_service
            .check(
                caller,
                Permission::ListPipelinesByProject(project_id.clone()),
            )
            .await?;
        self.pipeline_repo
            .list_by_project(project_id, pagination)
            .await
    }

    #[instrument(skip(self, caller), fields(org_id = %organization_id))]
    pub async fn list_by_organization(
        &self,
        caller: &CallerContext,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.permission_service
            .check(
                caller,
                Permission::ListPipelinesByOrganization(organization_id.clone()),
            )
            .await?;
        self.pipeline_repo
            .list_by_organization(organization_id, pagination)
            .await
    }

    /// Authorize + materialise the run of `pipeline_id` for `caller`. Loads
    /// the pipeline, mints a `Job`, persists it, and returns the dispatch
    /// payload. **Single** permission check (`Permission::RunPipeline`) — the
    /// internal repo calls deliberately bypass per-step Cedar so that
    /// granting "run" doesn't also require granting "get" and "create-job".
    /// The caller (handler) is responsible for actually publishing the
    /// dispatch to the broker.
    #[instrument(skip(self, caller), fields(pipeline_id = %pipeline_id))]
    pub async fn run(
        &self,
        caller: &CallerContext,
        pipeline_id: &PipelineId,
    ) -> DomainResult<(Job, JobDispatch)> {
        self.permission_service
            .check(caller, Permission::RunPipeline(pipeline_id.clone()))
            .await?;

        let pipeline = self.pipeline_repo.find_by_id(pipeline_id).await?;
        let job = Job::create_from_pipeline(&pipeline);
        let job = self.job_repo.create(&job).await?;
        let dispatch = JobDispatch {
            job_id: job.id().to_string(),
            pipeline_id: pipeline.id().to_string(),
            nodes: pipeline.nodes().to_vec(),
        };
        Ok((job, dispatch))
    }

    /// Record which agent the job was dispatched to. An internal continuation
    /// of the already-authorized `run` (the handler calls this once a agent
    /// accepts the dispatch), so it carries no extra Cedar check.
    #[instrument(skip(self), fields(job_id = %job_id, app_id = %app_id))]
    pub async fn assign_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()> {
        self.job_repo.set_agent(job_id, app_id).await
    }
}
