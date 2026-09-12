use crate::domain::errors::DomainResult;
use crate::domain::ids::TriggerId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Records inbound webhook deliveries for replay detection. A delivery is keyed
/// by `(trigger_id, delivery_id)`; the sender's delivery id when present, else a
/// stable digest of the request (the signature). Idempotent by construction.
#[async_trait]
pub trait TriggerDeliveryRepository {
    /// Atomically record a delivery, returning `true` if it is new (process it)
    /// or `false` if it was already seen (a replay — accept but do not re-fire).
    async fn record_or_detect(
        &self,
        trigger_id: &TriggerId,
        delivery_id: &str,
        received_at: DateTime<Utc>,
    ) -> DomainResult<bool>;
}
