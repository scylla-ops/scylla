use scylla_core::domain::entities::Agent;
use scylla_protocol::services::agent::AgentResponse;

pub fn agent_to_proto(agent: &Agent) -> AgentResponse {
    let status = if agent.is_connected() {
        "connected"
    } else {
        "disconnected"
    };
    AgentResponse {
        agent_id: agent.id().to_string(),
        hostname: agent.hostname().to_string(),
        status: status.to_string(),
        last_seen_at: agent.last_seen_at().to_rfc3339(),
        created_at: agent.created_at().to_rfc3339(),
        updated_at: agent.updated_at().to_rfc3339(),
    }
}
