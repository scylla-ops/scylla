use crate::application::dto::{GetJobRequestDto, JobResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::JobRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct GetJobUseCase<R>
where
    R: JobRepository + ?Sized,
{
    job_repo: Arc<R>,
}

impl<R> GetJobUseCase<R>
where
    R: JobRepository + ?Sized,
{
    pub async fn execute(&self, request: GetJobRequestDto) -> DomainResult<JobResponseDto> {
        let job = self.job_repo.find_by_id(&request.job_id).await?;

        Ok(JobResponseDto::from(&job))
    }
}
