use crate::application::permission::policy::{PolicyDefinition, PolicyRepository};
use crate::domain::entities::CedarPolicyId;
use crate::domain::errors::{DomainError, DomainResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
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
        let rows = sqlx::query!(
            "SELECT id, description, text, enabled, created_by, created_at, updated_at \
             FROM cedar_policies WHERE enabled = TRUE",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        Ok(rows
            .into_iter()
            .map(|r| {
                to_policy(
                    r.id,
                    r.description,
                    r.text,
                    r.enabled,
                    r.created_by,
                    r.created_at,
                    r.updated_at,
                )
            })
            .collect())
    }

    #[instrument(skip(self))]
    async fn list_all(&self) -> DomainResult<Vec<PolicyDefinition>> {
        let rows = sqlx::query!(
            "SELECT id, description, text, enabled, created_by, created_at, updated_at \
             FROM cedar_policies ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        Ok(rows
            .into_iter()
            .map(|r| {
                to_policy(
                    r.id,
                    r.description,
                    r.text,
                    r.enabled,
                    r.created_by,
                    r.created_at,
                    r.updated_at,
                )
            })
            .collect())
    }

    #[instrument(skip(self))]
    async fn get(&self, id: &CedarPolicyId) -> DomainResult<PolicyDefinition> {
        let row = sqlx::query!(
            "SELECT id, description, text, enabled, created_by, created_at, updated_at \
             FROM cedar_policies WHERE id = $1",
            id.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(infra)?;
        match row {
            Some(r) => Ok(to_policy(
                r.id,
                r.description,
                r.text,
                r.enabled,
                r.created_by,
                r.created_at,
                r.updated_at,
            )),
            None => Err(DomainError::not_found("CedarPolicy", id.as_str())),
        }
    }

    #[instrument(skip(self, policy), fields(policy_id = %policy.id))]
    async fn create(&self, policy: &PolicyDefinition) -> DomainResult<()> {
        sqlx::query!(
            "INSERT INTO cedar_policies \
             (id, description, text, enabled, created_by, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            policy.id.as_str(),
            policy.description,
            policy.text,
            policy.enabled,
            policy.created_by,
            policy.created_at,
            policy.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(infra)?;
        Ok(())
    }

    #[instrument(skip(self, policy), fields(policy_id = %policy.id))]
    async fn update(&self, policy: &PolicyDefinition) -> DomainResult<()> {
        sqlx::query!(
            "UPDATE cedar_policies \
             SET description = $2, text = $3, enabled = $4, updated_at = $5 \
             WHERE id = $1",
            policy.id.as_str(),
            policy.description,
            policy.text,
            policy.enabled,
            policy.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(infra)?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &CedarPolicyId) -> DomainResult<()> {
        sqlx::query!("DELETE FROM cedar_policies WHERE id = $1", id.as_str())
            .execute(&self.pool)
            .await
            .map_err(infra)?;
        Ok(())
    }
}

fn to_policy(
    id: String,
    description: String,
    text: String,
    enabled: bool,
    created_by: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> PolicyDefinition {
    PolicyDefinition {
        id: CedarPolicyId::new(id),
        description,
        text,
        enabled,
        created_by,
        created_at,
        updated_at,
    }
}

fn infra<E: std::fmt::Display>(e: E) -> DomainError {
    DomainError::Infrastructure(e.to_string())
}
