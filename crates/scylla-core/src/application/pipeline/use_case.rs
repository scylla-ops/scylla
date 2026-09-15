use crate::application::agent::dispatch::assemble_dispatch;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::application::{
    JobDispatch, JobRepository, PipelineRepository, ProjectRepository, SecretResolver,
};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, JobId, OrganizationId, PipelineId, ProjectId};
use crate::domain::job::Job;
use crate::domain::job::JobOrigin;
use crate::domain::permission::Permission;
use crate::domain::pipeline::PipelineName;
use crate::domain::pipeline::{Pipeline, PipelineNode};
use derive_more::Constructor;
use scylla_auth::authz::PermissionService;
use scylla_auth::caller::CallerContext;
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
    secret_resolver: Arc<dyn SecretResolver>,
}

impl<P: PipelineRepository, PR: ProjectRepository, J: JobRepository, PS: PermissionService>
    PipelineUseCases<P, PR, J, PS>
{
    #[instrument(skip_all, fields(name = %name, project_id = %project_id))]
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

    #[instrument(skip_all, fields(pipeline_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &PipelineId) -> DomainResult<Pipeline> {
        self.permission_service
            .check(caller, Permission::ReadPipeline(id.clone()))
            .await?;
        self.pipeline_repo.find_by_id(id).await
    }

    #[instrument(skip_all, fields(pipeline_id = %id))]
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

    #[instrument(skip_all, fields(pipeline_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &PipelineId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeletePipeline(id.clone()))
            .await?;
        self.pipeline_repo.find_by_id(id).await?;
        self.pipeline_repo.delete(id).await
    }

    #[instrument(skip(self, caller, pagination))]
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

    #[instrument(skip_all, fields(project_id = %project_id))]
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

    #[instrument(skip_all, fields(org_id = %organization_id))]
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

    /// One `RunPipeline` check; the repo calls bypass Cedar so "run" does not also require "get".
    pub async fn run(
        &self,
        caller: &CallerContext,
        pipeline_id: &PipelineId,
    ) -> DomainResult<(Job, JobDispatch)> {
        let origin = match caller {
            CallerContext::User(user_id) => JobOrigin::Human {
                user_id: user_id.clone(),
            },
            CallerContext::App(app_id) => JobOrigin::App {
                app_id: app_id.clone(),
            },
            CallerContext::Service(_) | CallerContext::Anonymous => {
                return Err(DomainError::forbidden(
                    "only a user or app can run a pipeline directly",
                ));
            }
        };
        self.run_with_inputs(caller, pipeline_id, &[], origin).await
    }

    #[instrument(skip_all, fields(pipeline_id = %pipeline_id, inputs = inputs.len()))]
    pub async fn run_with_inputs(
        &self,
        caller: &CallerContext,
        pipeline_id: &PipelineId,
        inputs: &[(String, String)],
        origin: JobOrigin,
    ) -> DomainResult<(Job, JobDispatch)> {
        self.permission_service
            .check(caller, Permission::RunPipeline(pipeline_id.clone()))
            .await?;

        let pipeline = self.pipeline_repo.find_by_id(pipeline_id).await?;
        let job = Job::create_from_pipeline(&pipeline, origin).with_inputs(inputs.to_vec());
        let job = self.job_repo.create(&job).await?;
        let dispatch =
            assemble_dispatch(&*self.pipeline_repo, &*self.secret_resolver, &job).await?;
        Ok((job, dispatch))
    }

    #[instrument(skip_all, fields(job_id = %job_id, app_id = %app_id))]
    pub async fn assign_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()> {
        self.job_repo.set_agent(job_id, app_id).await
    }
}
