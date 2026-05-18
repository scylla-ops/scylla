use crate::application::ports::AgentRepository;
use crate::domain::entities::{Agent, AgentId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::agent::Hostname;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::{DbFieldExt, SqlxResultExt};

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
    #[instrument(skip(self, agent), fields(agent_id = %agent.id()))]
    async fn create(&self, agent: &Agent) -> DomainResult<Agent> {
        queries::create(&self.pool, agent).await
    }

    #[instrument(skip(self), fields(agent_id = %id))]
    async fn find_by_id(&self, id: &AgentId) -> DomainResult<Agent> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip(self, agent), fields(agent_id = %agent.id()))]
    async fn update(&self, agent: &Agent) -> DomainResult<Agent> {
        queries::update(&self.pool, agent).await
    }

    #[instrument(skip(self), fields(agent_id = %id))]
    async fn delete(&self, id: &AgentId) -> DomainResult<()> {
        queries::delete(&self.pool, id).await
    }

    #[instrument(skip(self))]
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Agent>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_all(&self.pool).await?;
        let items = queries::list_page(&self.pool, &params).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self), fields(agent_id = %id))]
    async fn exists(&self, id: &AgentId) -> DomainResult<bool> {
        queries::exists(&self.pool, id).await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    fn row_into_agent(
        id: String,
        hostname: String,
        last_seen_at: DateTime<Utc>,
        shutdown_at: Option<DateTime<Utc>>,
        heartbeat_interval_secs: i64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Agent> {
        let hostname = Hostname::new(hostname).db_field("hostname")?;
        let heartbeat = u64::try_from(heartbeat_interval_secs).unwrap_or(0);
        Ok(Agent::from_persistence(
            AgentId::new(id),
            hostname,
            last_seen_at,
            shutdown_at,
            heartbeat,
            created_at,
            updated_at,
        ))
    }

    pub async fn create<'e, E>(executor: E, agent: &Agent) -> DomainResult<Agent>
    where
        E: PgExecutor<'e>,
    {
        let heartbeat = i64::try_from(agent.heartbeat_interval_secs()).unwrap_or(i64::MAX);
        sqlx::query!(
            r#"
            INSERT INTO agents (id, hostname, last_seen_at, shutdown_at, heartbeat_interval_secs, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            agent.id().as_str(),
            agent.hostname().as_str(),
            agent.last_seen_at(),
            agent.shutdown_at(),
            heartbeat,
            agent.created_at(),
            agent.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(agent.clone())
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &AgentId) -> DomainResult<Agent>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, hostname, last_seen_at, shutdown_at, heartbeat_interval_secs, created_at, updated_at
            FROM agents
            WHERE id = $1
            "#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("Agent", id.to_string())?;
        row_into_agent(
            rec.id,
            rec.hostname,
            rec.last_seen_at,
            rec.shutdown_at,
            rec.heartbeat_interval_secs,
            rec.created_at,
            rec.updated_at,
        )
    }

    pub async fn update<'e, E>(executor: E, agent: &Agent) -> DomainResult<Agent>
    where
        E: PgExecutor<'e>,
    {
        let heartbeat = i64::try_from(agent.heartbeat_interval_secs()).unwrap_or(i64::MAX);
        let res = sqlx::query!(
            r#"
            UPDATE agents
            SET hostname = $2,
                last_seen_at = $3,
                shutdown_at = $4,
                heartbeat_interval_secs = $5,
                updated_at = $6
            WHERE id = $1
            "#,
            agent.id().as_str(),
            agent.hostname().as_str(),
            agent.last_seen_at(),
            agent.shutdown_at(),
            heartbeat,
            agent.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        if res.rows_affected() == 0 {
            return Err(DomainError::not_found("Agent", agent.id().to_string()));
        }
        Ok(agent.clone())
    }

    pub async fn delete<'e, E>(executor: E, id: &AgentId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!("DELETE FROM agents WHERE id = $1", id.as_str())
            .execute(executor)
            .await
            .to_domain()?;
        Ok(())
    }

    pub async fn count_all<'e, E>(executor: E) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(r#"SELECT COUNT(*) AS "count!" FROM agents"#)
            .fetch_one(executor)
            .await
            .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_page<'e, E>(
        executor: E,
        params: &PaginationParams,
    ) -> DomainResult<Vec<Agent>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows = sqlx::query!(
            r#"
            SELECT id, hostname, last_seen_at, shutdown_at, heartbeat_interval_secs, created_at, updated_at
            FROM agents
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter()
            .map(|r| {
                row_into_agent(
                    r.id,
                    r.hostname,
                    r.last_seen_at,
                    r.shutdown_at,
                    r.heartbeat_interval_secs,
                    r.created_at,
                    r.updated_at,
                )
            })
            .collect()
    }

    pub async fn exists<'e, E>(executor: E, id: &AgentId) -> DomainResult<bool>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"SELECT EXISTS(SELECT 1 FROM agents WHERE id = $1) AS "exists!""#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(row.exists)
    }
}
