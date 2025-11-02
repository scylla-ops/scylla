use crate::application::dto::{DeletePipelineRequestDto, DeletePipelineResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::PipelineRepository;
use std::sync::Arc;

pub struct DeletePipelineUseCase<R>
where
    R: PipelineRepository + ?Sized,
{
    pipeline_repo: Arc<R>,
}

impl<R> DeletePipelineUseCase<R>
where
    R: PipelineRepository + ?Sized,
{
    pub fn new(pipeline_repo: Arc<R>) -> Self {
        Self { pipeline_repo }
    }

    pub async fn execute(
        &self,
        request: DeletePipelineRequestDto,
    ) -> DomainResult<DeletePipelineResponseDto> {
        let _ = self.pipeline_repo.find_by_id(&request.pipeline_id).await?;

        self.pipeline_repo.delete(&request.pipeline_id).await?;
        Ok(DeletePipelineResponseDto {})
    }
}
