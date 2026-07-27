use crate::application::authz::Visibility;
use crate::application::authz::grant::Grant;
use crate::domain::entities::{OrganizationId, Project, ProjectId, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait ProjectRepository {
    async fn create(&self, project: &Project) -> DomainResult<Project>;

    /// Insert a project together with its owner's grant in a single transaction,
    /// so a new project is never left without an owner. The owner is
    /// `grant.principal`; there is no separate membership row to write.
    async fn provision_with_owner(&self, project: &Project, grant: &Grant) -> DomainResult<()>;

    /// The users on the project: everyone holding a grant scoped to it, most
    /// recently granted first. Holders of an organization-wide grant are not
    /// listed — they administer the organization rather than work here.
    async fn list_principals(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>>;

    /// The projects a user works on: those they hold a grant on, plus every
    /// project of an organization they hold a grant on.
    async fn list_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>>;

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

    /// The organization's projects, narrowed to what `visible` allows. The
    /// filter is applied in SQL rather than after the fact, so the page size and
    /// the total count describe the same, already-narrowed set.
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
        visible: &Visibility,
    ) -> DomainResult<PaginatedResult<Project>>;

    async fn count_by_organization(&self, organization_id: &OrganizationId) -> DomainResult<u64>;
}
