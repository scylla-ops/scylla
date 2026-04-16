use crate::application::ports::AgentRepository;
use crate::domain::entities::{Agent, AgentId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use tracing::instrument;

pub struct SurrealAgentRepository {
    db: Surreal<Any>,
}

impl SurrealAgentRepository {
    #[must_use]
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AgentRepository for SurrealAgentRepository {
    #[instrument(skip(self, agent), fields(agent_id = %agent.id()))]
    async fn create(&self, agent: &Agent) -> DomainResult<Agent> {
        let db = self.db.clone();
        let agent = agent.clone();
        let created: Option<Agent> = db
            .create(RecordId::new(AgentId::table_name(), agent.id().as_str()))
            .content(agent.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        created
            .ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
    }

    #[instrument(skip(self), fields(agent_id = %id))]
    async fn find_by_id(&self, id: &AgentId) -> DomainResult<Agent> {
        let db = self.db.clone();
        let id = id.clone();
        let result: Option<Agent> = db
            .select(RecordId::new(AgentId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        result.ok_or_else(|| DomainError::not_found("Agent", id.to_string()))
    }

    #[instrument(skip(self, agent), fields(agent_id = %agent.id()))]
    async fn update(&self, agent: &Agent) -> DomainResult<Agent> {
        let db = self.db.clone();
        let agent = agent.clone();
        let updated: Option<Agent> = db
            .update(RecordId::new(AgentId::table_name(), agent.id().as_str()))
            .content(agent.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        updated.ok_or_else(|| DomainError::not_found("Agent", agent.id().to_string()))
    }

    #[instrument(skip(self), fields(agent_id = %id))]
    async fn delete(&self, id: &AgentId) -> DomainResult<()> {
        let db = self.db.clone();
        let id = id.clone();
        db.delete::<Option<Agent>>(RecordId::new(AgentId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Agent>> {
        let db = self.db.clone();
        let params = pagination.copied().unwrap_or_default();
        let table = AgentId::table_name().to_string();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) GROUP ALL")
            .bind(("table", table.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let agents: Vec<Agent> = db
            .query("SELECT * FROM type::table($table) ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("table", table))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(agents, &params, total_count))
    }

    #[instrument(skip(self), fields(agent_id = %id))]
    async fn exists(&self, id: &AgentId) -> DomainResult<bool> {
        let db = self.db.clone();
        let result: Option<Agent> = db
            .select(RecordId::new(AgentId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;
        Ok(result.is_some())
    }
}
