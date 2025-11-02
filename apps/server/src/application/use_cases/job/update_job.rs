use crate::application::dto::{JobResponseDto, UpdateJobRequestDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::JobRepository;
use std::sync::Arc;

pub struct UpdateJobUseCase<R>
where
    R: JobRepository + ?Sized,
{
    job_repo: Arc<R>,
}

impl<R> UpdateJobUseCase<R>
where
    R: JobRepository + ?Sized,
{
    pub fn new(job_repo: Arc<R>) -> Self {
        Self { job_repo }
    }

    pub async fn execute(&self, request: UpdateJobRequestDto) -> DomainResult<JobResponseDto> {
        let mut job_draft = self.job_repo.find_by_id(&request.job_id).await?;

        if let Some(status) = request.status {
            job_draft.update_status(status)?;
        }
        let updated_job = self.job_repo.update(&job_draft).await?;

        Ok(JobResponseDto::from(updated_job))
    }
}
