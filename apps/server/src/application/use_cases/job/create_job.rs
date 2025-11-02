use crate::application::dto::{CreateJobRequestDto, JobResponseDto};
use crate::domain::entities::Job;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::{JobRepository, PipelineRepository};
use std::sync::Arc;

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
    pub fn new(job_repo: Arc<R>, pipeline_repo: Arc<P>) -> Self {
        Self {
            job_repo,
            pipeline_repo,
        }
    }

    pub async fn execute(&self, request: CreateJobRequestDto) -> DomainResult<JobResponseDto> {
        let pipeline = self.pipeline_repo.find_by_id(&request.pipeline_id).await?;

        let job_draft = Job::create(request.pipeline_id, pipeline.content().to_owned())?;
        let created_job = self.job_repo.create(&job_draft).await?;

        Ok(JobResponseDto::from(created_job))
    }
}
