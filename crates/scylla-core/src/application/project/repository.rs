use crate::domain::entities::{OrganizationId, Project, ProjectId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait ProjectRepository {
    async fn create(&self, project: &Project) -> DomainResult<Project>;

    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project>;

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
