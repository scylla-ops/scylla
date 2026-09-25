//! The agent's reads. One block per query, in the order it runs: the struct, its access, its
//! output type, what `Fetch` reads.

use super::AgentUseCases;
use crate::application::AgentStats;
use crate::domain::agent::AgentHost;
use crate::domain::app::App;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, OrganizationId};
use crate::domain::permission::Permission;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla_extension::{Access, Authorized, Describe, Fetch, Fetched, Query, Run};
use std::collections::HashSet;

pub struct AgentView {
    pub app: App,
    pub connected: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub in_flight: usize,
    pub host: Option<AgentHost>,
}

#[derive(Debug)]
pub struct ListAgents {
    pub organization_id: OrganizationId,
}

impl Describe for ListAgents {
    fn access(&self) -> Access {
        Access::Requires(Permission::ListAgents(self.organization_id.clone()))
    }
}

impl Query for ListAgents {
    type Output = Vec<AgentView>;
}

#[async_trait]
impl Run<Fetch<ListAgents>> for AgentUseCases {
    async fn run(&self, input: Authorized<ListAgents>) -> DomainResult<Fetched<ListAgents>> {
        let agents = self
            .agent_repo
            .list_by_organization(&input.command().organization_id)
            .await?;
        let connected: HashSet<String> = self
            .registry
            .connected()
            .into_iter()
            .map(|id| id.as_str().to_string())
            .collect();

        let mut views = Vec::with_capacity(agents.len());
        for agent in &agents {
            let app = self.app_repo.find_by_id(agent.app_id()).await?;
            let is_connected = connected.contains(app.id().as_str());
            let in_flight = self.registry.in_flight(agent.app_id());
            views.push(AgentView {
                app,
                connected: is_connected,
                last_seen: agent.last_seen(),
                in_flight,
                host: agent.host().cloned(),
            });
        }
        Ok(input.fetched(views))
    }
}

#[derive(Debug)]
pub struct GetAgent {
    pub id: AppId,
}

impl Describe for GetAgent {
    fn access(&self) -> Access {
        Access::Requires(Permission::ReadApp(self.id.clone()))
    }
}

impl Query for GetAgent {
    type Output = AgentView;
}

#[async_trait]
impl Run<Fetch<GetAgent>> for AgentUseCases {
    async fn run(&self, input: Authorized<GetAgent>) -> DomainResult<Fetched<GetAgent>> {
        let id = &input.command().id;
        let agent = self.agent_repo.find_by_app_id(id).await?;
        let app = self.app_repo.find_by_id(id).await?;
        let connected = self
            .registry
            .connected()
            .iter()
            .any(|c| c.as_str() == id.as_str());
        let view = AgentView {
            app,
            connected,
            last_seen: agent.last_seen(),
            in_flight: self.registry.in_flight(id),
            host: agent.host().cloned(),
        };
        Ok(input.fetched(view))
    }
}

#[derive(Debug)]
pub struct GetAgentStats {
    pub id: AppId,
}

impl Describe for GetAgentStats {
    fn access(&self) -> Access {
        Access::Requires(Permission::ReadAppStats(self.id.clone()))
    }
}

impl Query for GetAgentStats {
    type Output = AgentStats;
}

#[async_trait]
impl Run<Fetch<GetAgentStats>> for AgentUseCases {
    async fn run(&self, input: Authorized<GetAgentStats>) -> DomainResult<Fetched<GetAgentStats>> {
        let stats = self.agent_repo.agent_stats(&input.command().id).await?;
        Ok(input.fetched(stats))
    }
}
