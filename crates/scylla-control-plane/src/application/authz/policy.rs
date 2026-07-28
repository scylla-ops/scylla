use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Control surface over the live authorization policy set. Implemented by the
/// Cedar adapter; the application uses it to atomically rebuild the live set
/// after a role or grant change (hot-reload). Kept Cedar-free so it lives in the
/// application layer.
#[async_trait]
pub trait PolicyControl: Send + Sync {
    /// Rebuild the live policy set from the stores (static base + roles + grants)
    /// and swap it in atomically. On failure the previous set is kept.
    async fn reload(&self) -> DomainResult<()>;
}
