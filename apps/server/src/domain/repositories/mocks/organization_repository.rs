//! Mock implementation for OrganizationRepository
//!
//! Provides mockall-based mocks for testing use cases that depend on OrganizationRepository.

use crate::domain::entities::Organization;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::OrganizationRepository as OrgRepoTrait;
use crate::domain::value_objects::{
    OrganizationId, OrganizationName, PaginatedResult, PaginationParams,
};
use mockall::mock;

/// Simplified trait for mocking (to work around mockall lifetime limitations)
#[async_trait::async_trait]
pub trait OrganizationRepositoryMock: Send + Sync {
    async fn create(&self, org: &Organization) -> DomainResult<Organization>;
    async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool>;
    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization>;
    async fn find_by_name(&self, name: &OrganizationName) -> DomainResult<Organization>;
    async fn update(&self, org: &Organization) -> DomainResult<Organization>;
    async fn delete(&self, id: &OrganizationId) -> DomainResult<()>;
}

mock! {
    pub OrganizationRepository {}

    #[async_trait::async_trait]
    impl OrganizationRepositoryMock for OrganizationRepository {
        async fn create(&self, org: &Organization) -> DomainResult<Organization>;
        async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool>;
        async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization>;
        async fn find_by_name(&self, name: &OrganizationName) -> DomainResult<Organization>;
        async fn update(&self, org: &Organization) -> DomainResult<Organization>;
        async fn delete(&self, id: &OrganizationId) -> DomainResult<()>;
    }
}

/// Adapter to make MockOrganizationRepository work with the actual OrganizationRepository trait
///
/// This adapter bridges the gap between the mockall-generated mock and the real trait,
/// handling methods that mockall can't mock directly (like those with lifetime parameters).
pub struct MockOrganizationRepositoryAdapter {
    pub inner: MockOrganizationRepository,
}

#[async_trait::async_trait]
impl OrgRepoTrait for MockOrganizationRepositoryAdapter {
    async fn create(&self, org: &Organization) -> DomainResult<Organization> {
        self.inner.create(org).await
    }

    async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool> {
        self.inner.name_exists(name).await
    }

    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization> {
        self.inner.find_by_id(id).await
    }

    async fn find_by_name(&self, name: &OrganizationName) -> DomainResult<Organization> {
        self.inner.find_by_name(name).await
    }

    async fn update(&self, org: &Organization) -> DomainResult<Organization> {
        self.inner.update(org).await
    }

    async fn delete(&self, id: &OrganizationId) -> DomainResult<()> {
        self.inner.delete(id).await
    }

    async fn list_all(
        &self,
        _pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        unimplemented!("list_all not commonly needed in use case tests")
    }

    async fn list_active(
        &self,
        _pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        unimplemented!("list_active not commonly needed in use case tests")
    }
}
