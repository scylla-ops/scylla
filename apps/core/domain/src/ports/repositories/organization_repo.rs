use crate::entities::{Organization, OrganizationId};
use crate::errors::DomainResult;
use crate::value_objects::organization::OrganizationName;
use crate::value_objects::{PaginatedResult, PaginationParams};

/// Repository trait for Organization entity
pub trait OrganizationRepository: Send + Sync {
    /// Create an organization
    fn create(
        &self,
        organization: &Organization,
    ) -> impl Future<Output = DomainResult<Organization>> + Send;

    /// Find an organization by ID
    fn find_by_id(
        &self,
        id: &OrganizationId,
    ) -> impl Future<Output = DomainResult<Organization>> + Send;

    /// Find an organization by name
    fn find_by_name(
        &self,
        name: &OrganizationName,
    ) -> impl Future<Output = DomainResult<Organization>> + Send;

    /// Update an organization
    fn update(
        &self,
        organization: &Organization,
    ) -> impl Future<Output = DomainResult<Organization>> + Send;

    /// Delete an organization by ID
    fn delete(&self, id: &OrganizationId) -> impl Future<Output = DomainResult<()>> + Send;

    /// List all organizations
    fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<Organization>>> + Send;

    /// List active organizations only
    fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<Organization>>> + Send;

    /// Check if an organization name exists
    fn name_exists(
        &self,
        name: &OrganizationName,
    ) -> impl Future<Output = DomainResult<bool>> + Send;
}
