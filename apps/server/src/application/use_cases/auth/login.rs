use crate::application::dto::{LoginRequestDto, LoginResponseDto};
use crate::application::ports::{AuthService, PasswordHasher};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::UserRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct LoginUseCase<R, H, A>
where
    R: UserRepository + ?Sized,
    H: PasswordHasher + ?Sized,
    A: AuthService + ?Sized,
{
    user_repo: Arc<R>,
    password_hasher: Arc<H>,
    auth_service: Arc<A>,
}

impl<R, H, A> LoginUseCase<R, H, A>
where
    R: UserRepository + ?Sized,
    H: PasswordHasher + ?Sized,
    A: AuthService + ?Sized,
{
    pub async fn execute(&self, request: LoginRequestDto) -> DomainResult<LoginResponseDto> {
        // Find user by username
        let user = self.user_repo.find_by_username(&request.username).await?;

        if !user.is_active() {
            return Err(DomainError::unauthorized("User account is inactive"));
        }

        // Verify password
        let is_valid = self
            .password_hasher
            .verify(&request.password, user.password_hash())
            .await?;

        if !is_valid {
            return Err(DomainError::unauthorized("Invalid username or password"));
        }

        // Generate token
        let token = self.auth_service.generate_token(user.id()).await?;

        Ok(LoginResponseDto {
            token,
            user_id: user.id().to_owned(),
        })
    }
}
