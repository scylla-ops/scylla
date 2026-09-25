pub mod commands;
pub mod queries;
pub mod repository;

pub use commands::{
    CreatePipeline, DeletePipeline, RunPipeline, RunPipelineWithInputs, UpdatePipeline,
};
pub use queries::{GetPipeline, ListOrganizationPipelines, ListPipelines, ListProjectPipelines};
pub use repository::PipelineRepository;

use crate::application::agent::dispatch::assemble_dispatch;
use crate::application::{DispatchUseCases, JobRepository, ProjectRepository, SecretResolver};
use crate::domain::errors::DomainResult;
use crate::domain::job::Job;
use derive_more::Constructor;
use std::sync::Arc;

/// The pipeline aggregate's stage runners, one block per action in `commands.rs` and
/// `queries.rs`. A run hands its job to an agent through `dispatch`, in its `commit` closure.
#[derive(Constructor)]
pub struct PipelineUseCases {
    pub(super) pipeline_repo: Arc<dyn PipelineRepository>,
    pub(super) project_repo: Arc<dyn ProjectRepository>,
    pub(super) job_repo: Arc<dyn JobRepository>,
    pub(super) secret_resolver: Arc<dyn SecretResolver>,
    pub(super) dispatch: Arc<DispatchUseCases>,
}

impl PipelineUseCases {
    async fn start(&self, job: &Job) -> DomainResult<Job> {
        let mut job = self.job_repo.create(job).await?;
        let dispatch =
            assemble_dispatch(&*self.pipeline_repo, &*self.secret_resolver, &job).await?;
        self.dispatch.place(&mut job, &dispatch).await;
        Ok(job)
    }
}

#[cfg(test)]
mod tests;
