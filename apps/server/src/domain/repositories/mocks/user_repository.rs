//! Mock implementation for UserRepository
//!
//! Provides mockall-based mocks for testing use cases that depend on UserRepository.

use crate::domain::entities::User;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::UserRepository as UserRepoTrait;
use crate::domain::value_objects::{PaginatedResult, PaginationParams, UserId, Username};
use mockall::mock;

/// Simplified trait for mocking (to work around mockall lifetime limitations)
#[async_trait::async_trait]
pub trait UserRepositoryMock: Send + Sync {
    async fn create(&self, user: &User) -> DomainResult<User>;
    async fn username_exists(&self, username: &Username) -> DomainResult<bool>;
    async fn find_by_id(&self, id: &UserId) -> DomainResult<User>;
    async fn find_by_username(&self, username: &Username) -> DomainResult<User>;
    async fn update(&self, user: &User) -> DomainResult<User>;
    async fn delete(&self, id: &UserId) -> DomainResult<()>;
}

mock! {
    pub UserRepository {}

    #[async_trait::async_trait]
    impl UserRepositoryMock for UserRepository {
        async fn create(&self, user: &User) -> DomainResult<User>;
        async fn username_exists(&self, username: &Username) -> DomainResult<bool>;
        async fn find_by_id(&self, id: &UserId) -> DomainResult<User>;
        async fn find_by_username(&self, username: &Username) -> DomainResult<User>;
        async fn update(&self, user: &User) -> DomainResult<User>;
        async fn delete(&self, id: &UserId) -> DomainResult<()>;
    }
}

/// Adapter to make MockUserRepository work with the actual UserRepository trait
///
/// This adapter bridges the gap between the mockall-generated mock and the real trait,
/// handling methods that mockall can't mock directly (like those with lifetime parameters).
pub struct MockUserRepositoryAdapter {
    pub inner: MockUserRepository,
}

#[async_trait::async_trait]
impl UserRepoTrait for MockUserRepositoryAdapter {
    async fn create(&self, user: &User) -> DomainResult<User> {
        self.inner.create(user).await
    }

    async fn username_exists(&self, username: &Username) -> DomainResult<bool> {
        self.inner.username_exists(username).await
    }

    async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
        self.inner.find_by_id(id).await
    }

    async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
        self.inner.find_by_username(username).await
    }

    async fn update(&self, user: &User) -> DomainResult<User> {
        self.inner.update(user).await
    }

    async fn delete(&self, id: &UserId) -> DomainResult<()> {
        self.inner.delete(id).await
    }

    async fn list_all(
        &self,
        _pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<User>> {
        unimplemented!("list_all not commonly needed in use case tests")
    }
}
