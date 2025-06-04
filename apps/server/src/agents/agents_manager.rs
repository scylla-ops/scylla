use protocol::AgentStatus;
use protocol::uuid::Uuid;
use std::collections::HashMap;
use std::time::SystemTime;
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Agent with UUID {0} not found")]
    AgentNotFound(Uuid),
}

#[derive(Debug)]
pub struct Agent {
    uuid: Uuid,
    last_seen: SystemTime,
    status: AgentStatus,
    agent_tx: mpsc::Sender<protocol::Message>,
}

#[derive(Default, Debug)]
pub struct AgentsManager {
    agents: HashMap<Uuid, Agent>,
}

impl AgentsManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_agent(&mut self, uuid: Uuid, agent_tx: mpsc::Sender<protocol::Message>) {
        self.agents.insert(
            uuid,
            Agent {
                uuid,
                last_seen: SystemTime::now(),
                status: AgentStatus::Undefined,
                agent_tx,
            },
        );
    }

    pub fn remove_agent(&mut self, uuid: Uuid) {
        self.agents.remove(&uuid);
    }

    pub fn update_last_seen(&mut self, uuid: Uuid) -> Result<(), AgentError> {
        self.agents
            .get_mut(&uuid)
            .ok_or(AgentError::AgentNotFound(uuid))?
            .last_seen = SystemTime::now();
        Ok(())
    }

    pub fn update_status(&mut self, uuid: Uuid, status: AgentStatus) -> Result<(), AgentError> {
        let agent = self
            .agents
            .get_mut(&uuid)
            .ok_or(AgentError::AgentNotFound(uuid))?;

        agent.status = status;
        Ok(())
    }

    pub fn get_agent_tx(&self, uuid: &Uuid) -> Option<&mpsc::Sender<protocol::Message>> {
        self.agents.get(uuid).map(|agent| &agent.agent_tx)
    }

    pub fn get_agent_by_filter<F>(&self, filter: F) -> Vec<&Agent>
    where
        F: Fn(&&Agent) -> bool,
    {
        self.agents.values().filter(filter).collect()
    }
}

impl protocol::HasStatus for Agent {
    fn status(&self) -> AgentStatus {
        self.status
    }
}

impl protocol::HasUuid for Agent {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}
