use crate::application::agent::repository::{AgentRepository, AgentStats};
use crate::domain::entities::{Agent, AppId, OrganizationId};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::SqlxResultExt;

/// Insert a `agents` extension row on any executor (pool or transaction).
/// Shared by the pool-backed repo and the atomic `provision_agent` transaction.
pub async fn insert<'e, E>(executor: E, agent: &Agent) -> DomainResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query!(
        r#"
        INSERT INTO agents (app_id, last_seen, created_at)
        VALUES ($1, $2, $3)
        "#,
        agent.app_id().as_str(),
        agent.last_seen(),
        agent.created_at(),
    )
    .execute(executor)
    .await
    .to_domain()?;
    Ok(())
}

/// Persistence for the `agents` table — the 1:1 specialization marking an app
/// as a agent. Run stats are derived from the `jobs` table, not stored here.
#[derive(Clone)]
pub struct PgAgentRepository {
    pool: PgPool,
}

impl PgAgentRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AgentRepository for PgAgentRepository {
    #[instrument(skip(self), fields(app_id = %app_id))]
    async fn find_by_app_id(&self, app_id: &AppId) -> DomainResult<Agent> {
        let rec = sqlx::query!(
            r#"
            SELECT app_id, last_seen, created_at
            FROM agents
            WHERE app_id = $1
            "#,
            app_id.as_str(),
        )
        .fetch_one(&self.pool)
        .await
        .not_found_as("Agent", app_id.to_string())?;
        Ok(Agent::from_persistence(
            AppId::new(rec.app_id),
            rec.last_seen,
            rec.created_at,
        ))
    }

    #[instrument(skip(self), fields(org_id = %org_id))]
    async fn list_by_organization(&self, org_id: &OrganizationId) -> DomainResult<Vec<Agent>> {
        let rows = sqlx::query!(
            r#"
            SELECT w.app_id, w.last_seen, w.created_at
            FROM agents w
            JOIN apps a ON a.id = w.app_id
            WHERE a.organization_id = $1
            ORDER BY w.created_at DESC
            "#,
            org_id.as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .to_domain()?;
        Ok(rows
            .into_iter()
            .map(|r| Agent::from_persistence(AppId::new(r.app_id), r.last_seen, r.created_at))
            .collect())
    }

    #[instrument(skip(self), fields(app_id = %app_id))]
    async fn touch_last_seen(&self, app_id: &AppId, at: DateTime<Utc>) -> DomainResult<()> {
        // Upsert so a agent connecting without a row (legacy / pre-migration)
        // self-heals — presence must never depend on this table existing.
        sqlx::query!(
            r#"
            INSERT INTO agents (app_id, last_seen, created_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (app_id) DO UPDATE SET last_seen = $2
            "#,
            app_id.as_str(),
            at,
        )
        .execute(&self.pool)
        .await
        .to_domain()?;
        Ok(())
    }

    #[instrument(skip(self), fields(app_id = %app_id))]
    async fn agent_stats(&self, app_id: &AppId) -> DomainResult<AgentStats> {
        let rec = sqlx::query!(
            r#"
            SELECT
                COUNT(*)                                          AS "total!",
                COUNT(*) FILTER (WHERE status = 'pending')        AS "pending!",
                COUNT(*) FILTER (WHERE status = 'running')        AS "running!",
                COUNT(*) FILTER (WHERE status = 'completed')      AS "completed!",
                COUNT(*) FILTER (WHERE status = 'failed')         AS "failed!",
                COUNT(*) FILTER (WHERE status = 'cancelled')      AS "cancelled!",
                MAX(created_at)                                   AS "last_run_at"
            FROM jobs
            WHERE agent_app_id = $1
            "#,
            app_id.as_str(),
        )
        .fetch_one(&self.pool)
        .await
        .to_domain()?;
        Ok(AgentStats {
            total: rec.total,
            pending: rec.pending,
            running: rec.running,
            completed: rec.completed,
            failed: rec.failed,
            cancelled: rec.cancelled,
            last_run_at: rec.last_run_at,
        })
    }
}
