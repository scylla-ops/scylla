use crate::application::dto::{
    ListPipelinesRequestDto, ListPipelinesResponseDto, PipelineResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::PipelineRepository;
use std::sync::Arc;

pub struct ListPipelinesUseCase<R>
where
    R: PipelineRepository + ?Sized,
{
    pipeline_repo: Arc<R>,
}

impl<R> ListPipelinesUseCase<R>
where
    R: PipelineRepository + ?Sized,
{
    pub fn new(pipeline_repo: Arc<R>) -> Self {
        Self { pipeline_repo }
    }

    pub async fn execute(
        &self,
        request: ListPipelinesRequestDto,
    ) -> DomainResult<ListPipelinesResponseDto> {
        let paginated_result = self
            .pipeline_repo
            .list_all(request.pagination.as_ref())
            .await?;
        let (pipelines, metadata) = paginated_result.into_parts();

        Ok(ListPipelinesResponseDto {
            pipelines: pipelines
                .into_iter()
                .map(PipelineResponseDto::from)
                .collect(),
            pagination: Some(metadata),
        })
    }
}
