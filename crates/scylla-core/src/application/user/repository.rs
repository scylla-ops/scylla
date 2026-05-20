use crate::domain::entities::{User, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::user::{Email, Username};
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository {
    async fn create(&self, user: &User) -> DomainResult<User>;

    async fn find_by_id(&self, id: &UserId) -> DomainResult<User>;

    async fn find_by_username(&self, username: &Username) -> DomainResult<User>;

    async fn find_by_email(&self, email: &Email) -> DomainResult<User>;

    async fn update(&self, user: &User) -> DomainResult<User>;

    async fn delete(&self, id: &UserId) -> DomainResult<()>;

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<User>>;

    async fn username_exists(&self, username: &Username) -> DomainResult<bool>;
}
