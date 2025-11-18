use crate::application::dto::{PipelineResponseDto, UpdatePipelineRequestDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::PipelineRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
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
