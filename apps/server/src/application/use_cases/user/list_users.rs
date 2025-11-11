use crate::application::dto::{ListUsersRequestDto, ListUsersResponseDto, UserResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::UserRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct ListUsersUseCase<R>
where
    R: UserRepository + ?Sized,
{
    user_repo: Arc<R>,
}

impl<R> ListUsersUseCase<R>
where
    R: UserRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: ListUsersRequestDto,
    ) -> DomainResult<ListUsersResponseDto> {
        let paginated_result = self.user_repo.list_all(request.pagination.as_ref()).await?;

        let (users, metadata) = paginated_result.into_parts();

        Ok(ListUsersResponseDto {
            users: users.iter().map(UserResponseDto::from).collect(),
            pagination: Some(metadata),
        })
    }
}
