use crate::domain::errors::DomainResult;
use async_trait::async_trait;

#[async_trait]
pub trait BlacklistRepository: Send + Sync {
    /// Check if an element is blacklisted.
    async fn is_blacklisted(&self, item: &str) -> DomainResult<bool>;

    /// Add an element to the blacklist.
    async fn add_to_blacklist(&self, item: String) -> DomainResult<()>;

    /// Delete an element from the blacklist.
    async fn remove_from_blacklist(&self, item: &str) -> DomainResult<()>;
}
