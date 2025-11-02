use crate::application::dto::{ProjectResponseDto, UpdateProjectRequestDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::ProjectRepository;

use std::sync::Arc;

pub struct UpdateProjectUseCase<R>
where
    R: ProjectRepository + ?Sized,
{
    project_repo: Arc<R>,
}

impl<R> UpdateProjectUseCase<R>
where
    R: ProjectRepository + ?Sized,
{
    pub fn new(project_repo: Arc<R>) -> Self {
        Self { project_repo }
    }

    pub async fn execute(
        &self,
        request: UpdateProjectRequestDto,
    ) -> DomainResult<ProjectResponseDto> {
        let mut project_draft = self.project_repo.find_by_id(&request.project_id).await?;

        if let Some(name) = request.name {
            project_draft.update_name(name)?;
        }

        if let Some(description) = request.description {
            project_draft.update_description(Some(description))?;
        }

        let updated_project = self.project_repo.update(&project_draft).await?;

        Ok(ProjectResponseDto::from(updated_project))
    }
}
