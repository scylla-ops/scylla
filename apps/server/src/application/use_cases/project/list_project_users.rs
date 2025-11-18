use crate::application::dto::{
    ListProjectUsersRequestDto, ListProjectUsersResponseDto, ProjectUserInfoResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::{UserProjectRepository, UserRepository};
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct ListProjectUsersUseCase<UP, U>
where
    UP: UserProjectRepository + ?Sized,
    U: UserRepository + ?Sized,
{
    user_project_repo: Arc<UP>,
    user_repo: Arc<U>,
}

impl<UP, U> ListProjectUsersUseCase<UP, U>
where
    UP: UserProjectRepository + ?Sized,
    U: UserRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: ListProjectUsersRequestDto,
    ) -> DomainResult<ListProjectUsersResponseDto> {
        let paginated_result = self
            .user_project_repo
            .list_users_in_project(&request.project_id, request.pagination.as_ref())
            .await?;

        let (user_ids, metadata) = paginated_result.into_parts();

        let mut users = Vec::with_capacity(user_ids.len());
        for user_id in user_ids {
            let relation = self
                .user_project_repo
                .find_by_user_and_project(&user_id, &request.project_id)
                .await?;
            let user = self.user_repo.find_by_id(&user_id).await?;
            users.push(ProjectUserInfoResponseDto {
                user_id,
                username: user.username().to_owned(),
                role: relation.role().to_owned(),
                joined_at: relation.joined_at(),
            });
        }

        Ok(ListProjectUsersResponseDto {
            users,
            pagination: Some(metadata),
        })
    }
}
