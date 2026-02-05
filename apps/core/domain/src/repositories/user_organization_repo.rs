use crate::entities::{OrganizationId, UserId, UserOrganization, UserOrganizationId};
use crate::errors::DomainResult;
use crate::value_objects::{PaginatedResult, PaginationParams};

/// Repository trait for UserOrganization entity
pub trait UserOrganizationRepository: Send + Sync {
    /// Create a user organization
    fn create(
        &self,
        user_organization: &UserOrganization,
    ) -> impl Future<Output = DomainResult<UserOrganization>> + Send;

    /// Find a user organization by ID
    fn find_by_id(
        &self,
        id: &UserOrganizationId,
    ) -> impl Future<Output = DomainResult<UserOrganization>> + Send;

    /// Find a user organization by user and organization
    fn find_by_user_and_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> impl Future<Output = DomainResult<UserOrganization>> + Send;

    /// Update a user organization
    fn update(
        &self,
        user_organization: &UserOrganization,
    ) -> impl Future<Output = DomainResult<UserOrganization>> + Send;

    /// Delete a user organization
    fn delete(&self, id: &UserOrganizationId) -> impl Future<Output = DomainResult<()>> + Send;

    /// List all user organizations
    fn list_all(&self) -> impl Future<Output = DomainResult<Vec<UserOrganization>>> + Send;

    /// List organizations for a user
    fn list_organizations_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<OrganizationId>>> + Send;

    /// List users in an organization
    fn list_users_in_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<UserId>>> + Send;

    /// Add a user to an organization
    fn add_user_to_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
        role: &str,
    ) -> impl Future<Output = DomainResult<UserOrganizationId>> + Send;

    /// Remove a user from an organization
    fn remove_user_from_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> impl Future<Output = DomainResult<()>> + Send;
}
