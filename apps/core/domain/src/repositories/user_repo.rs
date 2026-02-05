use crate::entities::{User, UserId};
use crate::errors::DomainResult;
use crate::value_objects::user::UserName;
use crate::value_objects::{PaginatedResult, PaginationParams};

/// Repository trait for User entity
pub trait UserRepository: Send + Sync {
    /// Create a user
    fn create(&self, user: &User) -> impl Future<Output = DomainResult<User>> + Send;

    /// Find a user by ID
    fn find_by_id(&self, id: &UserId) -> impl Future<Output = DomainResult<User>> + Send;

    /// Find a user by username
    fn find_by_username(
        &self,
        username: &UserName,
    ) -> impl Future<Output = DomainResult<User>> + Send;

    /// Update a user
    fn update(&self, user: &User) -> impl Future<Output = DomainResult<User>> + Send;

    /// Delete a user by ID
    fn delete(&self, id: &UserId) -> impl Future<Output = DomainResult<()>> + Send;

    /// List all users
    fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<User>>> + Send;

    /// Check if a username already exists
    fn username_exists(
        &self,
        username: &UserName,
    ) -> impl Future<Output = DomainResult<bool>> + Send;
}
