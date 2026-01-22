use crate::application::dto::{CreateJobRequestDto, JobResponseDto};
use crate::domain::entities::Job;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::{JobRepository, PipelineRepository};
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct CreateJobUseCase<R, P>
where
    R: JobRepository + ?Sized,
    P: PipelineRepository + ?Sized,
{
    job_repo: Arc<R>,
    pipeline_repo: Arc<P>,
}

impl<R, P> CreateJobUseCase<R, P>
where
    R: JobRepository + ?Sized,
    P: PipelineRepository + ?Sized,
{
    pub async fn execute(&self, request: CreateJobRequestDto) -> DomainResult<JobResponseDto> {
        // Récupérer la pipeline par ID (String maintenant)
        let pipeline = self.pipeline_repo.find_by_id(&request.pipeline_id).await?;

        // Créer le job à partir de la pipeline
        let job_draft = Job::create_from_pipeline(&pipeline)?;

        // Sauvegarder le job
        let created_job = self.job_repo.create(&job_draft).await?;

        Ok(JobResponseDto::from(created_job))
    }
}
