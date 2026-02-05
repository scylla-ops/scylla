use crate::entities::{Project, ProjectId};
use crate::errors::DomainResult;
use crate::value_objects::{PaginatedResult, PaginationParams};

/// Repository trait for Project entity
pub trait ProjectRepository: Send + Sync {
    /// Create a project
    fn create(&self, project: &Project) -> impl Future<Output = DomainResult<Project>> + Send;

    /// Find a project by ID
    fn find_by_id(&self, id: &ProjectId) -> impl Future<Output = DomainResult<Project>> + Send;

    /// Update a project
    fn update(&self, project: &Project) -> impl Future<Output = DomainResult<Project>> + Send;

    /// Delete a project by ID
    fn delete(&self, id: &ProjectId) -> impl Future<Output = DomainResult<()>> + Send;

    /// List all projects
    fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<Project>>> + Send;

    /// List active projects only
    fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<Project>>> + Send;
}
