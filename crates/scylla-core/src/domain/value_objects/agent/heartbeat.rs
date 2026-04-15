use serde::{Deserialize, Serialize};

/// Heartbeat published by the agent on `scylla.agents.heartbeat.{agent_id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHeartbeat {
    pub agent_id: String,
    pub hostname: String,
}

/// Graceful shutdown signal published by the agent on
/// `scylla.agents.shutdown.{agent_id}` when the process terminates cleanly.
/// Lets the recorder flip the agent to disconnected without waiting for the
/// heartbeat threshold to elapse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentShutdown {
    pub agent_id: String,
}
