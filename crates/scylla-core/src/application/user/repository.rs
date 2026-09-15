use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::DomainResult;
use crate::domain::ids::UserId;
use crate::domain::user::User;
use crate::domain::user::{Email, Username};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository {
    async fn create(&self, user: &User) -> DomainResult<User>;

    async fn find_by_id(&self, id: &UserId) -> DomainResult<User>;

    async fn find_by_ids(&self, ids: &[UserId]) -> DomainResult<Vec<User>>;

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
