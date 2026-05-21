use crate::application::permission::grant::Grant;
use crate::domain::entities::{Organization, OrganizationId, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::organization::OrganizationName;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait OrganizationRepository {
    async fn create(&self, organization: &Organization) -> DomainResult<Organization>;

    /// Insert an org together with the creator's membership and owner grant in a
    /// single transaction, so a new org is never left without an owner (a partial
    /// failure rolls the whole thing back).
    async fn provision_with_owner(
        &self,
        organization: &Organization,
        owner: &UserId,
        grant: &Grant,
    ) -> DomainResult<()>;

    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization>;

    async fn find_by_name(&self, name: &OrganizationName) -> DomainResult<Organization>;

    async fn update(&self, organization: &Organization) -> DomainResult<Organization>;

    async fn delete(&self, id: &OrganizationId) -> DomainResult<()>;

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>>;

    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>>;

    async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool>;
}
