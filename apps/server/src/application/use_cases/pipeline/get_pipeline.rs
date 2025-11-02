use crate::application::dto::{GetPipelineRequestDto, PipelineResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::PipelineRepository;
use std::sync::Arc;

pub struct GetPipelineUseCase<R>
where
    R: PipelineRepository + ?Sized,
{
    pipeline_repo: Arc<R>,
}

impl<R> GetPipelineUseCase<R>
where
    R: PipelineRepository + ?Sized,
{
    pub fn new(pipeline_repo: Arc<R>) -> Self {
        Self { pipeline_repo }
    }

    pub async fn execute(
        &self,
        request: GetPipelineRequestDto,
    ) -> DomainResult<PipelineResponseDto> {
        let pipeline = self.pipeline_repo.find_by_id(&request.pipeline_id).await?;

        Ok(PipelineResponseDto::from(pipeline))
    }
}
