use crate::domain::entities::User;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams, UserId, Username};
use async_trait::async_trait;

/// Repository trait for User entity
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Create a user
    async fn create(&self, user: &User) -> DomainResult<User>;

    /// Find a user by ID
    async fn find_by_id(&self, id: &UserId) -> DomainResult<User>;

    /// Find a user by username
    async fn find_by_username(&self, username: &Username) -> DomainResult<User>;

    /// Update a user
    async fn update(&self, user: &User) -> DomainResult<User>;

    /// Delete a user by ID
    async fn delete(&self, id: &UserId) -> DomainResult<()>;

    /// List all users
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<User>>;

    /// Check if a username already exists
    async fn username_exists(&self, username: &Username) -> DomainResult<bool>;
}
