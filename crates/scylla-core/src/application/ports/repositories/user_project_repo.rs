use crate::domain::entities::{ProjectId, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait UserProjectRepository {
    async fn add_member(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<()>;

    async fn remove_member(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<()>;

    async fn is_member(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<bool>;

    async fn list_members(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>>;

    async fn list_user_projects(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<ProjectId>>;
}
