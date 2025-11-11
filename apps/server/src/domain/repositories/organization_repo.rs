use crate::domain::entities::Organization;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{
    OrganizationId, OrganizationName, PaginatedResult, PaginationParams,
};
use async_trait::async_trait;

/// Repository trait for Organization entity
#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    /// Create an organization
    async fn create(&self, organization: &Organization) -> DomainResult<Organization>;

    /// Find an organization by ID
    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization>;

    /// Find an organization by name
    async fn find_by_name(&self, name: &OrganizationName) -> DomainResult<Organization>;

    /// Update an organization
    async fn update(&self, organization: &Organization) -> DomainResult<Organization>;

    /// Delete an organization by ID
    async fn delete(&self, id: &OrganizationId) -> DomainResult<()>;

    /// List all organizations
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>>;

    /// List active organizations only
    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>>;

    /// Check if an organization name exists
    async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool>;
}
