use super::Grant;
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence for explicit scoped grants. Read at `CedarPermissionService`
/// construction to link template instances; mutated by `GrantUseCases`.
#[async_trait]
pub trait GrantRepository: Send + Sync {
    async fn list_all(&self) -> DomainResult<Vec<Grant>>;
    async fn create(&self, grant: &Grant) -> DomainResult<()>;
    async fn delete(&self, id: &str) -> DomainResult<()>;
}
