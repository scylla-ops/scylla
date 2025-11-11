use crate::application::dto::{JobResponseDto, ListJobsRequestDto, ListJobsResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::JobRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct ListJobsUseCase<R>
where
    R: JobRepository + ?Sized,
{
    job_repo: Arc<R>,
}

impl<R> ListJobsUseCase<R>
where
    R: JobRepository + ?Sized,
{
    pub async fn execute(&self, request: ListJobsRequestDto) -> DomainResult<ListJobsResponseDto> {
        let paginated_result = self.job_repo.list_all(request.pagination.as_ref()).await?;
        let (jobs, metadata) = paginated_result.into_parts();

        Ok(ListJobsResponseDto {
            jobs: jobs.into_iter().map(JobResponseDto::from).collect(),
            pagination: Some(metadata),
        })
    }
}
