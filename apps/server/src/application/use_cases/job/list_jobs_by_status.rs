use crate::application::dto::{
    JobResponseDto, ListJobsByStatusRequestDto, ListJobsByStatusResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::JobRepository;
use std::sync::Arc;

pub struct ListJobsByStatusUseCase<R>
where
    R: JobRepository + ?Sized,
{
    job_repo: Arc<R>,
}

impl<R> ListJobsByStatusUseCase<R>
where
    R: JobRepository + ?Sized,
{
    pub fn new(job_repo: Arc<R>) -> Self {
        Self { job_repo }
    }

    pub async fn execute(
        &self,
        request: ListJobsByStatusRequestDto,
    ) -> DomainResult<ListJobsByStatusResponseDto> {
        let paginated_result = self
            .job_repo
            .list_by_status(&request.status, request.pagination.as_ref())
            .await?;
        let (jobs, metadata) = paginated_result.into_parts();

        Ok(ListJobsByStatusResponseDto {
            jobs: jobs.into_iter().map(JobResponseDto::from).collect(),
            pagination: Some(metadata),
        })
    }
}
