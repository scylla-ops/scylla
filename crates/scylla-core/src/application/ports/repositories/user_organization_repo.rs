use crate::domain::entities::{OrganizationId, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait UserOrganizationRepository {
    async fn add_member(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<()>;

    async fn remove_member(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<()>;

    async fn is_member(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<bool>;

    async fn list_members(
        &self,
        org_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>>;

    async fn list_user_organizations(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<OrganizationId>>;
}
