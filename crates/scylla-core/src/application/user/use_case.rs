use crate::application::{HashService, UserRepository};
use crate::domain::entities::{User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::user::{Password, Username};
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct UserUseCases<U: UserRepository, H: HashService> {
    user_repo: Arc<U>,
    hash_service: Arc<H>,
}

impl<U: UserRepository, H: HashService> UserUseCases<U, H> {
    #[instrument(skip(self, password), fields(username = %username))]
    pub async fn create(&self, username: Username, password: Password) -> DomainResult<User> {
        if self.user_repo.username_exists(&username).await? {
            return Err(DomainError::conflict("Username already exists"));
        }

        let password_hash = self.hash_service.hash(&password).await?;
        let user = User::create(username, password_hash);
        self.user_repo.create(&user).await
    }

    #[instrument(skip(self), fields(user_id = %id))]
    pub async fn get(&self, id: &UserId) -> DomainResult<User> {
        self.user_repo.find_by_id(id).await
    }

    #[instrument(skip(self), fields(user_id = %id))]
    pub async fn update(&self, id: &UserId, username: Option<Username>) -> DomainResult<User> {
        let mut user = self.user_repo.find_by_id(id).await?;

        if let Some(new_username) = username {
            if self.user_repo.username_exists(&new_username).await?
                && user.username() != &new_username
            {
                return Err(DomainError::conflict("Username already exists"));
            }
            user.update_username(new_username)?;
        }

        self.user_repo.update(&user).await
    }

    #[instrument(skip(self), fields(user_id = %id))]
    pub async fn delete(&self, id: &UserId) -> DomainResult<()> {
        self.user_repo.find_by_id(id).await?;
        self.user_repo.delete(id).await
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<User>> {
        self.user_repo.list_all(pagination).await
    }
}
