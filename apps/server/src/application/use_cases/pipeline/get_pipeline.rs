use crate::application::dto::{GetPipelineRequestDto, PipelineResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::PipelineRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
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
    pub async fn execute(
        &self,
        request: GetPipelineRequestDto,
    ) -> DomainResult<PipelineResponseDto> {
        let pipeline = self.pipeline_repo.find_by_id(&request.pipeline_id).await?;

        Ok(PipelineResponseDto::from(pipeline))
    }
}
