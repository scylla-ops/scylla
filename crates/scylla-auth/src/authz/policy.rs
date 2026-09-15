use crate::domain::errors::DomainResult;
use async_trait::async_trait;

#[async_trait]
pub trait PolicyControl: Send + Sync {
    async fn reload(&self) -> DomainResult<()>;
}
