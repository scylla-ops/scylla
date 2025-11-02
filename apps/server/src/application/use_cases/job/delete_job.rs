use crate::application::dto::{DeleteJobRequestDto, DeleteJobResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::JobRepository;
use std::sync::Arc;

pub struct DeleteJobUseCase<R>
where
    R: JobRepository + ?Sized,
{
    job_repo: Arc<R>,
}

impl<R> DeleteJobUseCase<R>
where
    R: JobRepository + ?Sized,
{
    pub fn new(job_repo: Arc<R>) -> Self {
        Self { job_repo }
    }

    pub async fn execute(
        &self,
        request: DeleteJobRequestDto,
    ) -> DomainResult<DeleteJobResponseDto> {
        let _ = self.job_repo.find_by_id(&request.job_id).await?;

        self.job_repo.delete(&request.job_id).await?;
        Ok(DeleteJobResponseDto {})
    }
}
