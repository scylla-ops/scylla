use crate::application::agent::dispatch::assemble_dispatch;
use crate::application::caller::CallerContext;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::application::{
    JobDispatch, JobRepository, PermissionService, PipelineRepository, ProjectRepository,
    SecretResolver,
};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, JobId, OrganizationId, PipelineId, ProjectId};
use crate::domain::job::Job;
use crate::domain::job::JobOrigin;
use crate::domain::permission::Permission;
use crate::domain::pipeline::PipelineName;
use crate::domain::pipeline::{Pipeline, PipelineNode};
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

    // Note: previously had `find_internal` for orchestration paths to bypass the
    // get-check; replaced by the consolidated `run()` method below which does a
    // single `Permission::RunPipeline` check and reads the pipeline via the repo.

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

    /// Authorize + materialise the run of `pipeline_id` for `caller`. Loads the
    /// pipeline, mints a `Job`, persists it, and returns the dispatch payload.
    /// **Single** permission check (`Permission::RunPipeline`) — the internal
    /// repo calls deliberately bypass per-step Cedar so granting "run" doesn't
    /// also require "get" and "create-job". The caller (handler / firing engine)
    /// then hands the payload to an agent via `DispatchUseCases` (in-process; no
    /// broker).
    pub async fn run(
        &self,
        caller: &CallerContext,
        pipeline_id: &PipelineId,
    ) -> DomainResult<(Job, JobDispatch)> {
        // A direct run is attributed to its caller: a human (`User`) or a machine
        // principal (`App`). `Service` / `Anonymous` never originate a run — the
        // permission check would reject them anyway, but we fail fast and clearly.
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

    /// Like [`run`](Self::run) but overlays `inputs` — already-resolved
    /// `(key, value)` env pairs, e.g. from a trigger — onto every node as
    /// literal (unmasked) env, merged AFTER secret resolution. A node's own env
    /// wins on a key collision, and inputs are plain literals that can never
    /// reference a secret. Same single `RunPipeline` check as `run`.
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
        // The job IS the run: it carries its inputs and origin, so the dispatch can
        // be (re)assembled identically whether placed now or retried later.
        let job = Job::create_from_pipeline(&pipeline, origin).with_inputs(inputs.to_vec());
        let job = self.job_repo.create(&job).await?;
        let dispatch =
            assemble_dispatch(&*self.pipeline_repo, &*self.secret_resolver, &job).await?;
        Ok((job, dispatch))
    }

    /// Record which agent the job was dispatched to. An internal continuation
    /// of the already-authorized `run` (the handler calls this once an agent
    /// accepts the dispatch), so it carries no extra Cedar check.
    #[instrument(skip_all, fields(job_id = %job_id, app_id = %app_id))]
    pub async fn assign_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()> {
        self.job_repo.set_agent(job_id, app_id).await
    }
}
