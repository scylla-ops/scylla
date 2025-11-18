use crate::application::dto::{
    JobResponseDto, ListJobsByPipelineRequestDto, ListJobsByPipelineResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::JobRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct ListJobsByPipelineUseCase<R>
where
    R: JobRepository + ?Sized,
{
    job_repo: Arc<R>,
}

impl<R> ListJobsByPipelineUseCase<R>
where
    R: JobRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: ListJobsByPipelineRequestDto,
    ) -> DomainResult<ListJobsByPipelineResponseDto> {
        let paginated_result = self
            .job_repo
            .list_by_pipeline(&request.pipeline_id, request.pagination.as_ref())
            .await?;
        let (jobs, metadata) = paginated_result.into_parts();

        Ok(ListJobsByPipelineResponseDto {
            jobs: jobs.into_iter().map(JobResponseDto::from).collect(),
            pagination: Some(metadata),
        })
    }
}
