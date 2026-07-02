use crate::application::authz::grant::Grant;
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

    /// Remove a member and, in the same transaction, strip everything they held
    /// under the org: grants scoped to the org itself, grants scoped to any of
    /// its projects, and their project memberships there. The Cedar member
    /// guard already denies an ex-member, but the rows must go too, or
    /// re-adding the user later would silently restore their old authority.
    /// Callers guard the scope's last human owner (see
    /// [`crate::application::authz::grant::removal_orphans_scope`]) and reload
    /// the policy set afterwards.
    async fn remove_member_and_grants(
        &self,
        user_id: &UserId,
        org_id: &OrganizationId,
    ) -> DomainResult<()>;

    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization>;

    /// Resolve many organizations in one round-trip (order unspecified, missing
    /// ids absent). Callers needing the input order re-associate by id. Avoids
    /// the N+1 of `find_by_id` in a loop.
    async fn find_by_ids(&self, ids: &[OrganizationId]) -> DomainResult<Vec<Organization>>;

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
