//! Mock implementation for UserOrganizationRepository
//!
//! Provides mockall-based mocks for testing use cases that depend on UserOrganizationRepository.

use crate::domain::entities::UserOrganization;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::UserOrganizationRepository as UserOrgRepoTrait;
use crate::domain::value_objects::{
    OrganizationId, PaginatedResult, PaginationParams, UserId, UserOrganizationId,
};
use mockall::mock;

/// Simplified trait for mocking (to work around mockall lifetime limitations)
#[async_trait::async_trait]
pub trait UserOrganizationRepositoryMock: Send + Sync {
    async fn create(&self, user_org: &UserOrganization) -> DomainResult<UserOrganization>;
    async fn find_by_id(&self, id: &UserOrganizationId) -> DomainResult<UserOrganization>;
    async fn find_by_user_and_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> DomainResult<UserOrganization>;
    async fn update(&self, user_org: &UserOrganization) -> DomainResult<UserOrganization>;
    async fn delete(&self, id: &UserOrganizationId) -> DomainResult<()>;
    async fn list_all(&self) -> DomainResult<Vec<UserOrganization>>;
    async fn add_user_to_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
        role: &str,
    ) -> DomainResult<UserOrganizationId>;
    async fn remove_user_from_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> DomainResult<()>;
}

mock! {
    pub UserOrganizationRepository {}

    #[async_trait::async_trait]
    impl UserOrganizationRepositoryMock for UserOrganizationRepository {
        async fn create(&self, user_org: &UserOrganization) -> DomainResult<UserOrganization>;
        async fn find_by_id(&self, id: &UserOrganizationId) -> DomainResult<UserOrganization>;
        async fn find_by_user_and_organization(
            &self,
            user_id: &UserId,
            organization_id: &OrganizationId,
        ) -> DomainResult<UserOrganization>;
        async fn update(&self, user_org: &UserOrganization) -> DomainResult<UserOrganization>;
        async fn delete(&self, id: &UserOrganizationId) -> DomainResult<()>;
        async fn list_all(&self) -> DomainResult<Vec<UserOrganization>>;
        async fn add_user_to_organization(
            &self,
            user_id: &UserId,
            organization_id: &OrganizationId,
            role: &str,
        ) -> DomainResult<UserOrganizationId>;
        async fn remove_user_from_organization(
            &self,
            user_id: &UserId,
            organization_id: &OrganizationId,
        ) -> DomainResult<()>;
    }
}

/// Adapter to make MockUserOrganizationRepository work with the actual UserOrganizationRepository trait
pub struct MockUserOrganizationRepositoryAdapter {
    pub inner: MockUserOrganizationRepository,
}

#[async_trait::async_trait]
impl UserOrgRepoTrait for MockUserOrganizationRepositoryAdapter {
    async fn create(&self, user_org: &UserOrganization) -> DomainResult<UserOrganization> {
        self.inner.create(user_org).await
    }

    async fn find_by_id(&self, id: &UserOrganizationId) -> DomainResult<UserOrganization> {
        self.inner.find_by_id(id).await
    }

    async fn find_by_user_and_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> DomainResult<UserOrganization> {
        self.inner
            .find_by_user_and_organization(user_id, organization_id)
            .await
    }

    async fn update(&self, user_org: &UserOrganization) -> DomainResult<UserOrganization> {
        self.inner.update(user_org).await
    }

    async fn delete(&self, id: &UserOrganizationId) -> DomainResult<()> {
        self.inner.delete(id).await
    }

    async fn list_all(&self) -> DomainResult<Vec<UserOrganization>> {
        self.inner.list_all().await
    }

    async fn list_organizations_for_user(
        &self,
        _user_id: &UserId,
        _pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<OrganizationId>> {
        unimplemented!("list_organizations_for_user not commonly needed in use case tests")
    }

    async fn list_users_in_organization(
        &self,
        _organization_id: &OrganizationId,
        _pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        unimplemented!("list_users_in_organization not commonly needed in use case tests")
    }

    async fn add_user_to_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
        role: &str,
    ) -> DomainResult<UserOrganizationId> {
        self.inner
            .add_user_to_organization(user_id, organization_id, role)
            .await
    }

    async fn remove_user_from_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> DomainResult<()> {
        self.inner
            .remove_user_from_organization(user_id, organization_id)
            .await
    }
}
