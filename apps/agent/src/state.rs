use protocol::AgentStatus;
use protocol::uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct ClientState {
    pub agent_id: Option<Uuid>,
    pub status: AgentStatus,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            agent_id: None,
            status: AgentStatus::Undefined,
        }
    }

    pub fn set_agent_id(&mut self, agent_id: Uuid) {
        debug!("ID assigned: {}", agent_id);
        self.agent_id = Some(agent_id);
    }

    pub fn set_status(&mut self, status: AgentStatus) {
        self.status = status;
    }
}

pub type SharedClientState = Arc<Mutex<ClientState>>;

pub fn new_shared_state() -> SharedClientState {
    Arc::new(Mutex::new(ClientState::new()))
}
