use crate::application::dto::{DeleteProjectRequestDto, DeleteProjectResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::ProjectRepository;
use std::sync::Arc;

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
    pub fn new(project_repo: Arc<R>) -> Self {
        Self { project_repo }
    }

    pub async fn execute(
        &self,
        request: DeleteProjectRequestDto,
    ) -> DomainResult<DeleteProjectResponseDto> {
        let _ = self.project_repo.find_by_id(&request.project_id).await?;

        self.project_repo.delete(&request.project_id).await?;
        Ok(DeleteProjectResponseDto {})
    }
}
