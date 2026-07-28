use crate::application::TriggerDeliveryRepository;
use crate::domain::errors::DomainResult;
use crate::domain::ids::TriggerId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::SqlxResultExt;

#[derive(Clone)]
pub struct PgTriggerDeliveryRepository {
    pool: PgPool,
}

impl PgTriggerDeliveryRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TriggerDeliveryRepository for PgTriggerDeliveryRepository {
    #[instrument(skip(self), fields(trigger_id = %trigger_id, delivery_id))]
    async fn record_or_detect(
        &self,
        trigger_id: &TriggerId,
        delivery_id: &str,
        received_at: DateTime<Utc>,
    ) -> DomainResult<bool> {
        queries::record_or_detect(&self.pool, trigger_id, delivery_id, received_at).await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    /// Insert the delivery; `ON CONFLICT DO NOTHING` makes a replay a no-op.
    /// `rows_affected() == 1` means new (process it), `0` means already seen.
    pub async fn record_or_detect<'e, E>(
        executor: E,
        trigger_id: &TriggerId,
        delivery_id: &str,
        received_at: DateTime<Utc>,
    ) -> DomainResult<bool>
    where
        E: PgExecutor<'e>,
    {
        let res = sqlx::query!(
            r#"
            INSERT INTO trigger_deliveries (trigger_id, delivery_id, received_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (trigger_id, delivery_id) DO NOTHING
            "#,
            trigger_id.as_str(),
            delivery_id,
            received_at,
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(res.rows_affected() > 0)
    }
}
