use crate::application::dto::{UpdateUserRequestDto, UserResponseDto};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::UserRepository;
use std::sync::Arc;

pub struct UpdateUserUseCase<R>
where
    R: UserRepository + ?Sized,
{
    user_repo: Arc<R>,
}

impl<R> UpdateUserUseCase<R>
where
    R: UserRepository + ?Sized,
{
    pub fn new(user_repo: Arc<R>) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, request: UpdateUserRequestDto) -> DomainResult<UserResponseDto> {
        let mut user_draft = self.user_repo.find_by_id(&request.user_id).await?;

        if let Some(username) = request.username {
            // Check if new username is taken by another user
            if self.user_repo.username_exists(&username).await? {
                let existing_user = self.user_repo.find_by_username(&username).await?;
                if existing_user.id() != &request.user_id {
                    return Err(DomainError::conflict(format!(
                        "Username '{}' is already taken",
                        username
                    )));
                }
            }
            user_draft.update_username(username)?;
        }

        let updated_user = self.user_repo.update(&user_draft).await?;

        Ok(UserResponseDto::from(updated_user))
    }
}
