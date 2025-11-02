use crate::application::dto::{PipelineResponseDto, UpdatePipelineRequestDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::PipelineRepository;
use std::sync::Arc;

pub struct UpdatePipelineUseCase<R>
where
    R: PipelineRepository + ?Sized,
{
    pipeline_repo: Arc<R>,
}

impl<R> UpdatePipelineUseCase<R>
where
    R: PipelineRepository + ?Sized,
{
    pub fn new(pipeline_repo: Arc<R>) -> Self {
        Self { pipeline_repo }
    }

    pub async fn execute(
        &self,
        request: UpdatePipelineRequestDto,
    ) -> DomainResult<PipelineResponseDto> {
        let mut pipeline_draft = self.pipeline_repo.find_by_id(&request.pipeline_id).await?;
        pipeline_draft.update_content(request.content)?;

        let updated_pipeline = self.pipeline_repo.update(&pipeline_draft).await?;

        Ok(PipelineResponseDto::from(updated_pipeline))
    }
}
