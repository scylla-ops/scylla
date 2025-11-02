//! Mock implementation for ProjectRepository
//!
//! Provides mockall-based mocks for testing use cases that depend on ProjectRepository.

use crate::domain::entities::Project;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::ProjectRepository as ProjRepoTrait;
use crate::domain::value_objects::{PaginatedResult, PaginationParams, ProjectId};
use mockall::mock;

/// Simplified trait for mocking (to work around mockall lifetime limitations)
#[async_trait::async_trait]
pub trait ProjectRepositoryMock: Send + Sync {
    async fn create(&self, project: &Project) -> DomainResult<Project>;
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project>;
    async fn update(&self, project: &Project) -> DomainResult<Project>;
    async fn delete(&self, id: &ProjectId) -> DomainResult<()>;
}

mock! {
    pub ProjectRepository {}

    #[async_trait::async_trait]
    impl ProjectRepositoryMock for ProjectRepository {
        async fn create(&self, project: &Project) -> DomainResult<Project>;
        async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project>;
        async fn update(&self, project: &Project) -> DomainResult<Project>;
        async fn delete(&self, id: &ProjectId) -> DomainResult<()>;
    }
}

/// Adapter to make MockProjectRepository work with the actual ProjectRepository trait
///
/// This adapter bridges the gap between the mockall-generated mock and the real trait,
/// handling methods that mockall can't mock directly (like those with lifetime parameters).
pub struct MockProjectRepositoryAdapter {
    pub inner: MockProjectRepository,
}

#[async_trait::async_trait]
impl ProjRepoTrait for MockProjectRepositoryAdapter {
    async fn create(&self, project: &Project) -> DomainResult<Project> {
        self.inner.create(project).await
    }

    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        self.inner.find_by_id(id).await
    }

    async fn update(&self, project: &Project) -> DomainResult<Project> {
        self.inner.update(project).await
    }

    async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        self.inner.delete(id).await
    }

    async fn list_all(
        &self,
        _pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        unimplemented!("list_all not commonly needed in use case tests")
    }

    async fn list_active(
        &self,
        _pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        unimplemented!("list_active not commonly needed in use case tests")
    }
}
