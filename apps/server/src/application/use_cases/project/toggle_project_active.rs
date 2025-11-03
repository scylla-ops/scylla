use crate::application::dto::{ToggleProjectActiveRequestDto, ToggleProjectActiveResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::ProjectRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
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
