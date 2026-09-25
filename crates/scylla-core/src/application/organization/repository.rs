use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, UserId};
use crate::domain::organization::Organization;
use crate::domain::organization::OrganizationName;
use async_trait::async_trait;
use scylla_auth::authz::Grant;

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn create(&self, organization: &Organization) -> DomainResult<Organization>;

    async fn provision_with_owner(
        &self,
        organization: &Organization,
        grant: &Grant,
    ) -> DomainResult<()>;

    async fn list_principals(
        &self,
        org_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>>;

    /// System-scoped grants are not expanded: an operator is not a member of every organization.
    async fn list_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>>;

    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization>;

    async fn update(&self, organization: &Organization) -> DomainResult<Organization>;

    async fn delete(&self, id: &OrganizationId) -> DomainResult<()>;

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>>;

    async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool>;
}
