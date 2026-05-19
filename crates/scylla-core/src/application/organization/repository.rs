use crate::domain::entities::{Organization, OrganizationId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::organization::OrganizationName;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait OrganizationRepository {
    async fn create(&self, organization: &Organization) -> DomainResult<Organization>;

    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization>;

    async fn find_by_name(&self, name: &OrganizationName) -> DomainResult<Organization>;

    async fn update(&self, organization: &Organization) -> DomainResult<Organization>;

    async fn delete(&self, id: &OrganizationId) -> DomainResult<()>;

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>>;

    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>>;

    async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool>;
}
