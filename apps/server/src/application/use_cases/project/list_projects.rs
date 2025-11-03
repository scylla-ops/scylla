use crate::application::dto::{
    ListProjectsRequestDto, ListProjectsResponseDto, ProjectResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::ProjectRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct ListProjectsUseCase<R>
where
    R: ProjectRepository + ?Sized,
{
    project_repo: Arc<R>,
}

impl<R> ListProjectsUseCase<R>
where
    R: ProjectRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: ListProjectsRequestDto,
    ) -> DomainResult<ListProjectsResponseDto> {
        let paginated_result = self
            .project_repo
            .list_all(request.pagination.as_ref())
            .await?;
        let (projects, metadata) = paginated_result.into_parts();

        Ok(ListProjectsResponseDto {
            projects: projects.into_iter().map(ProjectResponseDto::from).collect(),
            pagination: Some(metadata),
        })
    }
}
