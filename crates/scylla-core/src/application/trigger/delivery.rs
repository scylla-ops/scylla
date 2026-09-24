use crate::domain::errors::DomainResult;
use crate::domain::ids::TriggerId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait TriggerDeliveryRepository: Send + Sync {
    async fn record_or_detect(
        &self,
        trigger_id: &TriggerId,
        delivery_id: &str,
        received_at: DateTime<Utc>,
    ) -> DomainResult<bool>;
}
