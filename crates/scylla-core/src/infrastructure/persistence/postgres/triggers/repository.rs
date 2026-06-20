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
}
