use crate::application::dto::{
    ListUserProjectsRequestDto, ListUserProjectsResponseDto, ProjectResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::{ProjectRepository, UserProjectRepository};
use std::sync::Arc;

pub struct ListUserProjectsUseCase<UP, P>
where
    UP: UserProjectRepository + ?Sized,
    P: ProjectRepository + ?Sized,
{
    user_project_repo: Arc<UP>,
    project_repo: Arc<P>,
}

impl<UP, P> ListUserProjectsUseCase<UP, P>
where
    UP: UserProjectRepository + ?Sized,
    P: ProjectRepository + ?Sized,
{
    pub fn new(user_project_repo: Arc<UP>, project_repo: Arc<P>) -> Self {
        Self {
            user_project_repo,
            project_repo,
        }
    }

    pub async fn execute(
        &self,
        request: ListUserProjectsRequestDto,
    ) -> DomainResult<ListUserProjectsResponseDto> {
        let paginated_result = self
            .user_project_repo
            .list_projects_for_user(&request.user_id, request.pagination.as_ref())
            .await?;

        let (project_ids, metadata) = paginated_result.into_parts();

        let mut projects = Vec::with_capacity(project_ids.len());
        for pid in project_ids {
            let project = self.project_repo.find_by_id(&pid).await?;
            projects.push(ProjectResponseDto::from(project));
        }

        Ok(ListUserProjectsResponseDto {
            projects,
            pagination: Some(metadata),
        })
    }
}
