use crate::application::dto::{GetProjectRequestDto, ProjectResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::ProjectRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct GetProjectUseCase<R>
where
    R: ProjectRepository + ?Sized,
{
    project_repo: Arc<R>,
}

impl<R> GetProjectUseCase<R>
where
    R: ProjectRepository + ?Sized,
{
    pub async fn execute(&self, request: GetProjectRequestDto) -> DomainResult<ProjectResponseDto> {
        let project = self.project_repo.find_by_id(&request.project_id).await?;

        Ok(ProjectResponseDto::from(project))
    }
}
