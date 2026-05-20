use crate::application::permission::policy::{PolicyDefinition, PolicyRepository};
use crate::domain::entities::CedarPolicyId;
use crate::domain::errors::{DomainError, DomainResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tracing::instrument;

/// Persistence for runtime Cedar policies (`cedar_policies` table). Read on every
/// live policy-set rebuild (`list_enabled`) and mutated by `PolicyUseCases`.
#[derive(Clone)]
pub struct PgPolicyRepository {
    pool: PgPool,
}

impl PgPolicyRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PolicyRepository for PgPolicyRepository {
    #[instrument(skip(self))]
    async fn list_enabled(&self) -> DomainResult<Vec<PolicyDefinition>> {
        let rows = sqlx::query(
            "SELECT id, description, text, enabled, created_by, created_at, updated_at \
             FROM cedar_policies WHERE enabled = TRUE",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        rows.iter().map(row_to_policy).collect()
    }

    #[instrument(skip(self))]
    async fn list_all(&self) -> DomainResult<Vec<PolicyDefinition>> {
        let rows = sqlx::query(
            "SELECT id, description, text, enabled, created_by, created_at, updated_at \
             FROM cedar_policies ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        rows.iter().map(row_to_policy).collect()
    }

    #[instrument(skip(self))]
    async fn get(&self, id: &CedarPolicyId) -> DomainResult<PolicyDefinition> {
        let row = sqlx::query(
            "SELECT id, description, text, enabled, created_by, created_at, updated_at \
             FROM cedar_policies WHERE id = $1",
        )
        .bind(id.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(infra)?;
        match row {
            Some(r) => row_to_policy(&r),
            None => Err(DomainError::not_found("CedarPolicy", id.as_str())),
        }
    }

    #[instrument(skip(self, policy), fields(policy_id = %policy.id))]
    async fn create(&self, policy: &PolicyDefinition) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO cedar_policies \
             (id, description, text, enabled, created_by, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(policy.id.as_str())
        .bind(&policy.description)
        .bind(&policy.text)
        .bind(policy.enabled)
        .bind(&policy.created_by)
        .bind(policy.created_at)
        .bind(policy.updated_at)
        .execute(&self.pool)
        .await
        .map_err(infra)?;
        Ok(())
    }

    #[instrument(skip(self, policy), fields(policy_id = %policy.id))]
    async fn update(&self, policy: &PolicyDefinition) -> DomainResult<()> {
        sqlx::query(
            "UPDATE cedar_policies \
             SET description = $2, text = $3, enabled = $4, updated_at = $5 \
             WHERE id = $1",
        )
        .bind(policy.id.as_str())
        .bind(&policy.description)
        .bind(&policy.text)
        .bind(policy.enabled)
        .bind(policy.updated_at)
        .execute(&self.pool)
        .await
        .map_err(infra)?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &CedarPolicyId) -> DomainResult<()> {
        sqlx::query("DELETE FROM cedar_policies WHERE id = $1")
            .bind(id.as_str())
            .execute(&self.pool)
            .await
            .map_err(infra)?;
        Ok(())
    }
}

fn row_to_policy(r: &sqlx::postgres::PgRow) -> DomainResult<PolicyDefinition> {
    Ok(PolicyDefinition {
        id: CedarPolicyId::new(r.try_get::<String, _>("id").map_err(infra)?),
        description: r.try_get("description").map_err(infra)?,
        text: r.try_get("text").map_err(infra)?,
        enabled: r.try_get("enabled").map_err(infra)?,
        created_by: r.try_get("created_by").map_err(infra)?,
        created_at: r.try_get::<DateTime<Utc>, _>("created_at").map_err(infra)?,
        updated_at: r.try_get::<DateTime<Utc>, _>("updated_at").map_err(infra)?,
    })
}

fn infra<E: std::fmt::Display>(e: E) -> DomainError {
    DomainError::Infrastructure(e.to_string())
}
