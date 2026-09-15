use super::{Grant, Principal, Scope};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

#[async_trait]
pub trait GrantRepository: Send + Sync {
    async fn list_all(&self) -> DomainResult<Vec<Grant>>;
    async fn create(&self, grant: &Grant) -> DomainResult<()>;
    async fn delete(&self, id: &str) -> DomainResult<()>;

    /// One statement: row by row would leave a window where part of the access survives.
    /// System-scoped grants are never touched, so an org admin cannot strip a platform operator.
    async fn revoke_all(&self, principal: &Principal, scope: &Scope) -> DomainResult<u64>;
}
