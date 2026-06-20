use crate::application::TriggerRepository;
use crate::domain::entities::{PipelineId, Trigger, TriggerId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::trigger::{TriggerInput, TriggerName, TriggerSource};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool, types::Json};
use tracing::instrument;

use super::super::error::{DbFieldExt, SqlxResultExt};

#[derive(Clone)]
pub struct PgTriggerRepository {
    pool: PgPool,
}

impl PgTriggerRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TriggerRepository for PgTriggerRepository {
    #[instrument(skip(self, trigger), fields(trigger_id = %trigger.id()))]
    async fn create(&self, trigger: &Trigger) -> DomainResult<Trigger> {
        queries::create(&self.pool, trigger).await
    }

    #[instrument(skip(self), fields(trigger_id = %id))]
    async fn find_by_id(&self, id: &TriggerId) -> DomainResult<Trigger> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip(self, trigger), fields(trigger_id = %trigger.id()))]
    async fn update(&self, trigger: &Trigger) -> DomainResult<Trigger> {
        queries::update(&self.pool, trigger).await
    }

    #[instrument(skip(self), fields(trigger_id = %id))]
    async fn delete(&self, id: &TriggerId) -> DomainResult<()> {
        queries::delete(&self.pool, id).await
    }

    #[instrument(skip(self), fields(pipeline_id = %pipeline_id))]
    async fn list_by_pipeline(&self, pipeline_id: &PipelineId) -> DomainResult<Vec<Trigger>> {
        queries::list_by_pipeline(&self.pool, pipeline_id).await
    }

    #[instrument(skip(self))]
    async fn list_unscheduled_cron(&self) -> DomainResult<Vec<Trigger>> {
        queries::list_unscheduled_cron(&self.pool).await
    }

    #[instrument(skip(self, compute_next), fields(now = %now, limit))]
    async fn claim_due_cron(
        &self,
        now: DateTime<Utc>,
        limit: i64,
        compute_next: &(dyn for<'a> Fn(&'a Trigger) -> DomainResult<DateTime<Utc>> + Sync),
    ) -> DomainResult<Vec<Trigger>> {
        let mut tx = self.pool.begin().await.to_domain()?;

        // Lock the due rows; SKIP LOCKED so a concurrent pass/instance never waits
        // on or double-claims the same trigger.
        let rows: Vec<TriggerRow> = sqlx::query_as!(
            TriggerRow,
            r#"
            SELECT id, pipeline_id, name,
                   source AS "source: Json<TriggerSource>",
                   inputs AS "inputs: Json<Vec<TriggerInput>>",
                   enabled, next_fire_at, last_fired_at, last_status, created_at, updated_at
            FROM pipeline_triggers
            WHERE enabled
              AND kind = 'cron'
              AND next_fire_at IS NOT NULL
              AND next_fire_at <= $1
            ORDER BY next_fire_at
            LIMIT $2
            FOR UPDATE SKIP LOCKED
            "#,
            now,
            limit,
        )
        .fetch_all(&mut *tx)
        .await
        .to_domain()?;

        let mut claimed = Vec::with_capacity(rows.len());
        for row in rows {
            let trigger = Trigger::try_from(row)?;
            // Advance to the next occurrence in the same tx so this occurrence is
            // consumed exactly once. A trigger whose expression won't compute is
            // left as-is and excluded (it was seeded valid, so this is defensive).
            let Ok(next) = compute_next(&trigger) else {
                continue;
            };
            sqlx::query!(
                r#"
                UPDATE pipeline_triggers
                SET next_fire_at = $2, updated_at = $3
                WHERE id = $1
                "#,
                trigger.id().as_str(),
                next,
                now,
            )
            .execute(&mut *tx)
            .await
            .to_domain()?;
            claimed.push(trigger);
        }

        tx.commit().await.to_domain()?;
        Ok(claimed)
    }
}

/// Row shape for `SELECT ... FROM pipeline_triggers`. The denormalized `kind`
/// column is written for indexing/routing but not read back — the source kind is
/// recovered from the `source` JSONB tag.
#[derive(sqlx::FromRow)]
struct TriggerRow {
    id: String,
    pipeline_id: String,
    name: String,
    source: Json<TriggerSource>,
    inputs: Json<Vec<TriggerInput>>,
    enabled: bool,
    next_fire_at: Option<DateTime<Utc>>,
    last_fired_at: Option<DateTime<Utc>>,
    last_status: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TriggerRow> for Trigger {
    type Error = DomainError;
    fn try_from(r: TriggerRow) -> DomainResult<Self> {
        let name = TriggerName::new(r.name).db_field("trigger name")?;
        Ok(Trigger::from_persistence(
            TriggerId::new(r.id),
            PipelineId::new(r.pipeline_id),
            name,
            r.source.0,
            r.inputs.0,
            r.enabled,
            r.next_fire_at,
            r.last_fired_at,
            r.last_status,
            r.created_at,
            r.updated_at,
        ))
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    pub async fn create<'e, E>(executor: E, trigger: &Trigger) -> DomainResult<Trigger>
    where
        E: PgExecutor<'e>,
    {
        let source = Json(trigger.source().clone());
        let inputs = Json(trigger.inputs().to_vec());
        sqlx::query!(
            r#"
            INSERT INTO pipeline_triggers
                (id, pipeline_id, name, kind, source, inputs, enabled,
                 next_fire_at, last_fired_at, last_status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            trigger.id().as_str(),
            trigger.pipeline_id().as_str(),
            trigger.name().as_str(),
            trigger.source().kind().as_str(),
            source as _,
            inputs as _,
            trigger.is_enabled(),
            trigger.next_fire_at(),
            trigger.last_fired_at(),
            trigger.last_status(),
            trigger.created_at(),
            trigger.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(trigger.clone())
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &TriggerId) -> DomainResult<Trigger>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query_as!(
            TriggerRow,
            r#"
            SELECT id, pipeline_id, name,
                   source AS "source: Json<TriggerSource>",
                   inputs AS "inputs: Json<Vec<TriggerInput>>",
                   enabled, next_fire_at, last_fired_at, last_status, created_at, updated_at
            FROM pipeline_triggers
            WHERE id = $1
            "#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("Trigger", id.to_string())?
        .try_into()
    }

    pub async fn update<'e, E>(executor: E, trigger: &Trigger) -> DomainResult<Trigger>
    where
        E: PgExecutor<'e>,
    {
        let source = Json(trigger.source().clone());
        let inputs = Json(trigger.inputs().to_vec());
        let res = sqlx::query!(
            r#"
            UPDATE pipeline_triggers
            SET name = $2,
                kind = $3,
                source = $4,
                inputs = $5,
                enabled = $6,
                next_fire_at = $7,
                last_fired_at = $8,
                last_status = $9,
                updated_at = $10
            WHERE id = $1
            "#,
            trigger.id().as_str(),
            trigger.name().as_str(),
            trigger.source().kind().as_str(),
            source as _,
            inputs as _,
            trigger.is_enabled(),
            trigger.next_fire_at(),
            trigger.last_fired_at(),
            trigger.last_status(),
            trigger.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        if res.rows_affected() == 0 {
            return Err(DomainError::not_found("Trigger", trigger.id().to_string()));
        }
        Ok(trigger.clone())
    }

    pub async fn delete<'e, E>(executor: E, id: &TriggerId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!("DELETE FROM pipeline_triggers WHERE id = $1", id.as_str())
            .execute(executor)
            .await
            .to_domain()?;
        Ok(())
    }

    pub async fn list_by_pipeline<'e, E>(
        executor: E,
        pipeline_id: &PipelineId,
    ) -> DomainResult<Vec<Trigger>>
    where
        E: PgExecutor<'e>,
    {
        let rows: Vec<TriggerRow> = sqlx::query_as!(
            TriggerRow,
            r#"
            SELECT id, pipeline_id, name,
                   source AS "source: Json<TriggerSource>",
                   inputs AS "inputs: Json<Vec<TriggerInput>>",
                   enabled, next_fire_at, last_fired_at, last_status, created_at, updated_at
            FROM pipeline_triggers
            WHERE pipeline_id = $1
            ORDER BY created_at
            "#,
            pipeline_id.as_str(),
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter().map(Trigger::try_from).collect()
    }

    /// Enabled cron triggers with no `next_fire_at` yet — the scheduler seeds them.
    pub async fn list_unscheduled_cron<'e, E>(executor: E) -> DomainResult<Vec<Trigger>>
    where
        E: PgExecutor<'e>,
    {
        let rows: Vec<TriggerRow> = sqlx::query_as!(
            TriggerRow,
            r#"
            SELECT id, pipeline_id, name,
                   source AS "source: Json<TriggerSource>",
                   inputs AS "inputs: Json<Vec<TriggerInput>>",
                   enabled, next_fire_at, last_fired_at, last_status, created_at, updated_at
            FROM pipeline_triggers
            WHERE enabled AND kind = 'cron' AND next_fire_at IS NULL
            ORDER BY created_at
            "#,
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter().map(Trigger::try_from).collect()
    }
}
