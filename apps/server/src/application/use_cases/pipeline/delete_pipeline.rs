use crate::application::dto::{DeletePipelineRequestDto, DeletePipelineResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::PipelineRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
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
    pub async fn execute(
        &self,
        request: DeletePipelineRequestDto,
    ) -> DomainResult<DeletePipelineResponseDto> {
        let _ = self.pipeline_repo.find_by_id(&request.pipeline_id).await?;

        self.pipeline_repo.delete(&request.pipeline_id).await?;
        Ok(DeletePipelineResponseDto {})
    }
}
