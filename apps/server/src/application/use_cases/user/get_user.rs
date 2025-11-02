use crate::application::dto::{GetUserRequestDto, UserResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::UserRepository;
use std::sync::Arc;

pub struct GetUserUseCase<R>
where
    R: UserRepository + ?Sized,
{
    user_repo: Arc<R>,
}

impl<R> GetUserUseCase<R>
where
    R: UserRepository + ?Sized,
{
    pub fn new(user_repo: Arc<R>) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, request: GetUserRequestDto) -> DomainResult<UserResponseDto> {
        let user = self.user_repo.find_by_id(&request.user_id).await?;

        Ok(UserResponseDto::from(user))
    }
}
