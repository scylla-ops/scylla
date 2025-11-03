use crate::application::dto::{CreatePipelineRequestDto, PipelineResponseDto};
use crate::domain::entities::Pipeline;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::PipelineRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct CreatePipelineUseCase<R>
where
    R: PipelineRepository + ?Sized,
{
    pipeline_repo: Arc<R>,
}

impl<R> CreatePipelineUseCase<R>
where
    R: PipelineRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: CreatePipelineRequestDto,
    ) -> DomainResult<PipelineResponseDto> {
        let pipeline_draft = Pipeline::create(request.content)?;
        let created_pipeline = self.pipeline_repo.create(&pipeline_draft).await?;

        Ok(PipelineResponseDto::from(created_pipeline))
    }
}
