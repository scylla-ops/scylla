use crate::entities::{ProjectId, UserId, UserProject, UserProjectId};
use crate::errors::DomainResult;
use crate::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

/// Repository trait for UserProject entity
#[async_trait]
pub trait UserProjectRepository {
    /// Create a user project
    async fn create(
        &self,
        user_project: &UserProject,
    ) -> DomainResult<UserProject>;

    /// Find a user project by ID
    async fn find_by_id(
        &self,
        id: &UserProjectId,
    ) -> DomainResult<UserProject>;

    /// Find a user project by user and project
    async fn find_by_user_and_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<UserProject>;

    /// Update a user project
    async fn update(
        &self,
        user_project: &UserProject,
    ) -> DomainResult<UserProject>;

    /// Delete a user project
    async fn delete(&self, id: &UserProjectId) -> DomainResult<()>;

    /// List all user projects
    async fn list_all(&self) -> DomainResult<Vec<UserProject>>;

    /// List projects for a user
    async fn list_projects_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<ProjectId>>;

    /// List users in a project
    async fn list_users_in_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>>;

    /// Add a user to a project
    async fn add_user_to_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        role: &str,
    ) -> DomainResult<UserProjectId>;

    /// Remove a user from a project
    async fn remove_user_from_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<()>;
}
