use crate::entities::{ProjectId, UserId, UserProject, UserProjectId};
use crate::errors::DomainResult;
use crate::value_objects::{PaginatedResult, PaginationParams};

/// Repository trait for UserProject entity
pub trait UserProjectRepository: Send + Sync {
    /// Create a user project
    fn create(
        &self,
        user_project: &UserProject,
    ) -> impl Future<Output = DomainResult<UserProject>> + Send;

    /// Find a user project by ID
    fn find_by_id(
        &self,
        id: &UserProjectId,
    ) -> impl Future<Output = DomainResult<UserProject>> + Send;

    /// Find a user project by user and project
    fn find_by_user_and_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> impl Future<Output = DomainResult<UserProject>> + Send;

    /// Update a user project
    fn update(
        &self,
        user_project: &UserProject,
    ) -> impl Future<Output = DomainResult<UserProject>> + Send;

    /// Delete a user project
    fn delete(&self, id: &UserProjectId) -> impl Future<Output = DomainResult<()>> + Send;

    /// List all user projects
    fn list_all(&self) -> impl Future<Output = DomainResult<Vec<UserProject>>> + Send;

    /// List projects for a user
    fn list_projects_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<ProjectId>>> + Send;

    /// List users in a project
    fn list_users_in_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<UserId>>> + Send;

    /// Add a user to a project
    fn add_user_to_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        role: &str,
    ) -> impl Future<Output = DomainResult<UserProjectId>> + Send;

    /// Remove a user from a project
    fn remove_user_from_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> impl Future<Output = DomainResult<()>> + Send;
}
