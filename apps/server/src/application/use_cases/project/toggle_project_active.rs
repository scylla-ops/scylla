use crate::application::dto::{ToggleProjectActiveRequestDto, ToggleProjectActiveResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::ProjectRepository;
use std::sync::Arc;

pub struct ToggleProjectActiveUseCase<R>
where
    R: ProjectRepository + ?Sized,
{
    project_repo: Arc<R>,
}

impl<R> ToggleProjectActiveUseCase<R>
where
    R: ProjectRepository + ?Sized,
{
    pub fn new(project_repo: Arc<R>) -> Self {
        Self { project_repo }
    }

    pub async fn execute(
        &self,
        request: ToggleProjectActiveRequestDto,
    ) -> DomainResult<ToggleProjectActiveResponseDto> {
        let mut project_draft = self.project_repo.find_by_id(&request.project_id).await?;

        project_draft.toggle_active()?;

        self.project_repo.update(&project_draft).await?;

        Ok(ToggleProjectActiveResponseDto {})
    }
}
