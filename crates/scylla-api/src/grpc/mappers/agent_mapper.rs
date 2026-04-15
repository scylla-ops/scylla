use chrono::Duration;
use protocol::services::agent::AgentResponse;
use scylla_core::domain::entities::Agent;

/// Threshold beyond which an agent's last heartbeat makes it disconnected.
/// Should match the recorder's `--agent-stale-after-secs`.
const STALE_AFTER_SECS: i64 = 15;

pub fn agent_to_proto(agent: &Agent) -> AgentResponse {
    let threshold = Duration::seconds(STALE_AFTER_SECS);
    let status = if agent.is_connected(threshold) {
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
