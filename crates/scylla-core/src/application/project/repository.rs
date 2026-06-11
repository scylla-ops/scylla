use crate::application::authz::grant::Grant;
use crate::domain::entities::{OrganizationId, Project, ProjectId, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait ProjectRepository {
    async fn create(&self, project: &Project) -> DomainResult<Project>;

    /// Insert a project together with the creator's membership and owner grant in
    /// a single transaction, so a new project is never left without an owner (a
    /// partial failure rolls the whole thing back). Mirrors
    /// [`OrganizationRepository::provision_with_owner`].
    async fn provision_with_owner(
        &self,
        project: &Project,
        owner: &UserId,
        grant: &Grant,
    ) -> DomainResult<()>;

    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project>;

    /// Resolve many projects in one round-trip (order unspecified, missing ids
    /// absent). Callers needing the input order re-associate by id. Avoids the
    /// N+1 of `find_by_id` in a loop.
    async fn find_by_ids(&self, ids: &[ProjectId]) -> DomainResult<Vec<Project>>;

    async fn update(&self, project: &Project) -> DomainResult<Project>;

    async fn delete(&self, id: &ProjectId) -> DomainResult<()>;

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>>;

    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>>;

    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>>;

    async fn count_by_organization(&self, organization_id: &OrganizationId) -> DomainResult<u64>;
}
