use crate::application::dto::{RunPipelineRequestDto, RunPipelineResponseDto};
use crate::domain::entities::Job;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::{JobRepository, PipelineRepository};
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct RunPipelineUseCase<R, P>
where
    R: JobRepository + ?Sized,
    P: PipelineRepository + ?Sized,
{
    job_repo: Arc<R>,
    pipeline_repo: Arc<P>,
}

impl<R, P> RunPipelineUseCase<R, P>
where
    R: JobRepository + ?Sized,
    P: PipelineRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: RunPipelineRequestDto,
    ) -> DomainResult<RunPipelineResponseDto> {
        let pipeline = self.pipeline_repo.find_by_id(&request.pipeline_id).await?;

        let job_draft = Job::create(request.pipeline_id, pipeline.content().clone().try_into()?)?;
        let created_job = self.job_repo.create(&job_draft).await?;

        // TODO: Cmon... Do something...
        // Should assign the job to a worker or something like that

        Ok(RunPipelineResponseDto {
            job_id: created_job.id().to_owned(),
        })
    }
}
