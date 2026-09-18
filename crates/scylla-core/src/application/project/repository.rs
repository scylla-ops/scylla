use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, ProjectId, UserId};
use crate::domain::project::Project;
use async_trait::async_trait;
use scylla_auth::authz::{Grant, Visibility};

#[async_trait]
pub trait ProjectRepository {
    async fn create(&self, project: &Project) -> DomainResult<Project>;

    async fn provision_with_owner(&self, project: &Project, grant: &Grant) -> DomainResult<()>;

    /// Organization-wide grant holders are not listed here.
    async fn list_principals(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>>;

    async fn list_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>>;

    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project>;

    async fn find_by_ids(&self, ids: &[ProjectId]) -> DomainResult<Vec<Project>>;

    /// Writes only if the row still carries `project.version()`, and returns the row with the
    /// bumped version. A stale value is `Conflict`; a missing row is `NotFound`.
    async fn update(&self, project: &Project) -> DomainResult<Project>;

    /// Same version rule as `update`.
    async fn delete(&self, project: &Project) -> DomainResult<()>;

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
        visible: &Visibility,
    ) -> DomainResult<PaginatedResult<Project>>;
}
