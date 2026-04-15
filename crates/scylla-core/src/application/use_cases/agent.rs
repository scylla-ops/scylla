use crate::application::ports::AgentRepository;
use crate::domain::entities::{Agent, AgentId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::agent::Hostname;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct AgentUseCases<A: AgentRepository> {
    agent_repo: Arc<A>,
}

impl<A: AgentRepository> AgentUseCases<A> {
    #[instrument(skip(self), fields(agent_id = %id))]
    pub async fn get(&self, id: &AgentId) -> DomainResult<Agent> {
        self.agent_repo.find_by_id(id).await
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Agent>> {
        self.agent_repo.list_all(pagination).await
    }

    #[instrument(skip(self), fields(agent_id = %id))]
    pub async fn delete(&self, id: &AgentId) -> DomainResult<()> {
        self.agent_repo.find_by_id(id).await?;
        self.agent_repo.delete(id).await
    }

    /// Record a heartbeat. Creates the row on first sight, otherwise refreshes
    /// `last_seen_at` + `hostname`.
    #[instrument(skip(self), fields(agent_id = %id))]
    pub async fn record_heartbeat(
        &self,
        id: &AgentId,
        hostname: Hostname,
    ) -> DomainResult<Agent> {
        match self.agent_repo.find_by_id(id).await {
            Ok(mut agent) => {
                agent.record_heartbeat(hostname);
                self.agent_repo.update(&agent).await
            }
            Err(_) => {
                let agent = Agent::create(id.clone(), hostname);
                self.agent_repo.create(&agent).await
            }
        }
    }

    /// Record a graceful shutdown. Stamps `shutdown_at` so derived status flips
    /// to disconnected on the next read; `last_seen_at` is preserved.
    #[instrument(skip(self), fields(agent_id = %id))]
    pub async fn record_shutdown(&self, id: &AgentId) -> DomainResult<Agent> {
        let mut agent = self.agent_repo.find_by_id(id).await?;
        agent.record_shutdown();
        self.agent_repo.update(&agent).await
    }
}
