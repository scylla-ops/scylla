use hermes_broker_client::Publisher;
use scylla_core::domain::value_objects::agent::{AgentHeartbeat, AgentShutdown};
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Channel;
use tracing::warn;

const HEARTBEAT_SUBJECT_PREFIX: &str = "scylla.agents.heartbeat";
const SHUTDOWN_SUBJECT_PREFIX: &str = "scylla.agents.shutdown";

/// Emits periodic heartbeats and a one-shot shutdown event.
pub struct PresencePublisher {
    publisher: Publisher,
    agent_id: String,
    hostname: String,
    heartbeat_interval_secs: u64,
}

impl PresencePublisher {
    #[must_use]
    pub fn new(
        channel: Channel,
        agent_id: String,
        hostname: String,
        heartbeat_interval_secs: u64,
    ) -> Self {
        Self {
            publisher: Publisher::new(channel),
            agent_id,
            hostname,
            heartbeat_interval_secs,
        }
    }

    pub async fn publish_heartbeat(&self) {
        let beat = AgentHeartbeat {
            agent_id: self.agent_id.clone(),
            hostname: self.hostname.clone(),
            heartbeat_interval_secs: self.heartbeat_interval_secs,
        };
        let payload = serde_json::to_vec(&beat).expect("serialization cannot fail");
        let subject = format!("{HEARTBEAT_SUBJECT_PREFIX}.{}", self.agent_id);
        if let Err(e) = self.publisher.publish(subject, payload).await {
            warn!(error = %e, "failed to publish heartbeat");
        }
    }

    pub async fn publish_shutdown(&self) {
        let msg = AgentShutdown {
            agent_id: self.agent_id.clone(),
        };
        let payload = serde_json::to_vec(&msg).expect("serialization cannot fail");
        let subject = format!("{SHUTDOWN_SUBJECT_PREFIX}.{}", self.agent_id);
        if let Err(e) = self.publisher.publish(subject, payload).await {
            warn!(error = %e, "failed to publish shutdown");
        }
    }

    /// Spawn a background ticker emitting heartbeats at the given interval.
    pub fn spawn_heartbeat_ticker(self: Arc<Self>, interval: Duration) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                self.publish_heartbeat().await;
            }
        });
    }
}
