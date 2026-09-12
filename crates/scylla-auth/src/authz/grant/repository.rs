use super::{Grant, Principal, Scope};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence for explicit scoped grants. Read at `CedarPermissionService`
/// construction to link template instances; mutated by `GrantUseCases`.
#[async_trait]
pub trait GrantRepository: Send + Sync {
    async fn list_all(&self) -> DomainResult<Vec<Grant>>;
    async fn create(&self, grant: &Grant) -> DomainResult<()>;
    async fn delete(&self, id: &str) -> DomainResult<()>;

    /// Strip every grant a principal holds at `scope` **and below it**, in one
    /// statement. This is the kill switch: removing someone from an
    /// organization has to reach their project grants too, and doing it row by
    /// row would leave a window where half their access survives.
    ///
    /// System-scoped grants are never touched, whatever the scope asked for, so
    /// an organization administrator cannot strip a platform operator.
    ///
    /// Returns how many grants were removed.
    async fn revoke_all(&self, principal: &Principal, scope: &Scope) -> DomainResult<u64>;
}
