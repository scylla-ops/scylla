pub mod commands;
pub mod queries;
pub mod repository;

pub use commands::{CreatePipeline, DeletePipeline, RunPipeline, UpdatePipeline};
pub use queries::{GetPipeline, ListOrganizationPipelines, ListPipelines, ListProjectPipelines};
pub use repository::PipelineRepository;

use crate::application::agent::dispatch::assemble_dispatch;
use crate::application::{JobDispatch, JobRepository, ProjectRepository, SecretResolver};
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, JobId, PipelineId};
use crate::domain::job::{Job, JobOrigin};
use crate::domain::permission::Permission;
use derive_more::Constructor;
use scylla_auth::authz::PermissionService;
use std::sync::Arc;
use tracing::instrument;

/// The pipeline aggregate's stage runners, one block per action in `commands.rs` and
/// `queries.rs`. `run_with_inputs` and `assign_agent` stay outside the pipeline: a trigger fire
/// calls them as its runner App, with an origin and inputs no RPC sends.
#[derive(Constructor)]
pub struct PipelineUseCases {
    pub(super) pipeline_repo: Arc<dyn PipelineRepository>,
    pub(super) project_repo: Arc<dyn ProjectRepository>,
    pub(super) job_repo: Arc<dyn JobRepository>,
    pub(super) permission_service: Arc<dyn PermissionService>,
    pub(super) secret_resolver: Arc<dyn SecretResolver>,
}

impl PipelineUseCases {
    /// One `RunPipeline` check; the repo calls bypass Cedar so "run" does not also require "get".
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
        self.start(&job).await
    }

    #[instrument(skip_all, fields(job_id = %job_id, app_id = %app_id))]
    pub async fn assign_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()> {
        self.job_repo.set_agent(job_id, app_id).await
    }

    pub(super) async fn start(&self, job: &Job) -> DomainResult<(Job, JobDispatch)> {
        let job = self.job_repo.create(job).await?;
        let dispatch =
            assemble_dispatch(&*self.pipeline_repo, &*self.secret_resolver, &job).await?;
        Ok((job, dispatch))
    }
}

#[cfg(test)]
mod tests;
