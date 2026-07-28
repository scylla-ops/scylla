use crate::application::authz::grant::Grant;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, UserId};
use crate::domain::organization::Organization;
use crate::domain::organization::OrganizationName;
use async_trait::async_trait;

#[async_trait]
pub trait OrganizationRepository {
    async fn create(&self, organization: &Organization) -> DomainResult<Organization>;

    /// Insert an org together with its owner's grant in a single transaction, so
    /// a new org is never left without an owner (a partial failure rolls the
    /// whole thing back). The owner is `grant.principal`; there is no separate
    /// membership row to write.
    async fn provision_with_owner(
        &self,
        organization: &Organization,
        grant: &Grant,
    ) -> DomainResult<()>;

    /// The users with access to the organization: everyone holding a grant on it
    /// or on one of its projects, most recently granted first. Distinct, so
    /// someone holding several roles appears once.
    async fn list_principals(
        &self,
        org_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>>;

    /// The organizations a user belongs to: those they hold a grant on, plus
    /// those owning a project they hold a grant on. System-scoped grants are not
    /// expanded — a platform operator is not a member of every organization.
    async fn list_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>>;

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
