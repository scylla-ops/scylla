use crate::domain::entities::UserOrganization;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{
    OrganizationId, PaginatedResult, PaginationParams, UserId, UserOrganizationId,
};
use async_trait::async_trait;

/// Repository trait for UserOrganization entity
#[async_trait]
pub trait UserOrganizationRepository: Send + Sync {
    /// Create a user organization
    async fn create(&self, user_organization: &UserOrganization) -> DomainResult<UserOrganization>;

    /// Find a user organization by ID
    async fn find_by_id(&self, id: &UserOrganizationId) -> DomainResult<UserOrganization>;

    /// Find a user organization by user and organization
    async fn find_by_user_and_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> DomainResult<UserOrganization>;

    /// Update a user organization
    async fn update(&self, user_organization: &UserOrganization) -> DomainResult<UserOrganization>;

    /// Delete a user organization
    async fn delete(&self, id: &UserOrganizationId) -> DomainResult<()>;

    /// List user organizations for a user
    async fn list_all(&self) -> DomainResult<Vec<UserOrganization>>;

    /// List organizations for a user
    async fn list_organizations_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<OrganizationId>>;

    /// List users in an organization
    async fn list_users_in_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>>;

    /// Add a user to an organization
    async fn add_user_to_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
        role: &str,
    ) -> DomainResult<UserOrganizationId>;

    /// Remove a user from an organization
    async fn remove_user_from_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> DomainResult<()>;
}
