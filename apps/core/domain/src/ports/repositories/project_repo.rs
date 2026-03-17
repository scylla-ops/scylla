use crate::entities::{Project, ProjectId};
use crate::errors::DomainResult;
use crate::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

/// Repository trait for Project entity
#[async_trait]
pub trait ProjectRepository {
    /// Create a project
    async fn create(&self, project: &Project) -> DomainResult<Project>;

    /// Find a project by ID
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project>;

    /// Update a project
    async fn update(&self, project: &Project) -> DomainResult<Project>;

    /// Delete a project by ID
    async fn delete(&self, id: &ProjectId) -> DomainResult<()>;

    /// List all projects
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>>;

    /// List active projects only
    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>>;
}
