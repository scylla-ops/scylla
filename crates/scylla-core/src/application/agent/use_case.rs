use crate::application::caller::CallerContext;
use crate::application::{AgentRepository, PermissionService};
use crate::domain::entities::{Agent, AgentId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::agent::Hostname;
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct AgentUseCases<A: AgentRepository, PS: PermissionService> {
    agent_repo: Arc<A>,
    permission_service: Arc<PS>,
}

impl<A: AgentRepository, PS: PermissionService> AgentUseCases<A, PS> {
    #[instrument(skip(self, caller), fields(agent_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &AgentId) -> DomainResult<Agent> {
        self.permission_service
            .check(caller, Permission::ReadAgent(id.clone()))
            .await?;
        self.agent_repo.find_by_id(id).await
    }

    #[instrument(skip(self, caller))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Agent>> {
        self.permission_service
            .check(caller, Permission::ListAgents)
            .await?;
        self.agent_repo.list_all(pagination).await
    }

    #[instrument(skip(self, caller), fields(agent_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &AgentId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeleteAgent(id.clone()))
            .await?;
        self.agent_repo.find_by_id(id).await?;
        self.agent_repo.delete(id).await
    }

    /// Record a heartbeat. Recorder-only path; routed through the trait so a
    /// future Service-specific Cedar allowlist can replace the blanket
    /// service-permit without touching call sites.
    #[instrument(skip(self, caller), fields(agent_id = %id))]
    pub async fn record_heartbeat(
        &self,
        caller: &CallerContext,
        id: &AgentId,
        hostname: Hostname,
        heartbeat_interval_secs: u64,
    ) -> DomainResult<Agent> {
        self.permission_service
            .check(caller, Permission::WriteAgent(id.clone()))
            .await?;

        match self.agent_repo.find_by_id(id).await {
            Ok(mut agent) => {
                agent.record_heartbeat(hostname, heartbeat_interval_secs);
                self.agent_repo.update(&agent).await
            }
            Err(DomainError::NotFound { .. }) => {
                let agent = Agent::create(id.clone(), hostname, heartbeat_interval_secs);
                self.agent_repo.create(&agent).await
            }
            Err(e) => Err(e),
        }
    }

    /// Record a graceful shutdown. Recorder-only path.
    #[instrument(skip(self, caller), fields(agent_id = %id))]
    pub async fn record_shutdown(
        &self,
        caller: &CallerContext,
        id: &AgentId,
    ) -> DomainResult<Agent> {
        self.permission_service
            .check(caller, Permission::WriteAgent(id.clone()))
            .await?;
        let mut agent = self.agent_repo.find_by_id(id).await?;
        agent.record_shutdown();
        self.agent_repo.update(&agent).await
    }
}
