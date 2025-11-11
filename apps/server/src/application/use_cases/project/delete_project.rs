use crate::application::dto::{DeleteProjectRequestDto, DeleteProjectResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::ProjectRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct DeleteProjectUseCase<R>
where
    R: ProjectRepository + ?Sized,
{
    project_repo: Arc<R>,
}

impl<R> DeleteProjectUseCase<R>
where
    R: ProjectRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: DeleteProjectRequestDto,
    ) -> DomainResult<DeleteProjectResponseDto> {
        let _ = self.project_repo.find_by_id(&request.project_id).await?;

        self.project_repo.delete(&request.project_id).await?;
        Ok(DeleteProjectResponseDto {})
    }
}
