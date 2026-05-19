use crate::error::ListenerError;
use hermes_broker_client::Subscriber;
use scylla_core::application::AgentUseCases;
use scylla_core::domain::entities::AgentId;
use scylla_core::domain::value_objects::agent::{AgentHeartbeat, AgentShutdown, Hostname};
use scylla_core::infrastructure::PgAgentRepository;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tonic::transport::Channel;
use tracing::{error, info, warn};

const HEARTBEAT_SUBJECT: &str = "scylla.agents.heartbeat.>";
const SHUTDOWN_SUBJECT: &str = "scylla.agents.shutdown.>";
const RECONNECT_BACKOFF: StdDuration = StdDuration::from_secs(2);

pub async fn run_heartbeat(channel: Channel, agent_uc: Arc<AgentUseCases<PgAgentRepository>>) {
    loop {
        if let Err(e) = heartbeat_once(channel.clone(), &agent_uc).await {
            warn!(error = %e, "heartbeat listener exited; reconnecting");
        }
        tokio::time::sleep(RECONNECT_BACKOFF).await;
    }
}

async fn heartbeat_once(
    channel: Channel,
    agent_uc: &AgentUseCases<PgAgentRepository>,
) -> Result<(), ListenerError> {
    let mut subscriber = Subscriber::new(channel)
        .await
        .map_err(|e| ListenerError::SubscriberInit(e.to_string()))?;
    subscriber
        .subscribe(HEARTBEAT_SUBJECT, None)
        .await
        .map_err(|e| ListenerError::Subscribe {
            subject: HEARTBEAT_SUBJECT.to_string(),
            message: e.to_string(),
        })?;
    info!(subject = HEARTBEAT_SUBJECT, "subscribed");

    while let Some(msg) = subscriber.recv().await {
        let beat: AgentHeartbeat = match serde_json::from_slice(&msg.payload) {
            Ok(b) => b,
            Err(e) => {
                warn!(error = %e, "bad heartbeat payload");
                continue;
            }
        };
        let hostname = match Hostname::new(&beat.hostname) {
            Ok(h) => h,
            Err(e) => {
                warn!(agent_id = %beat.agent_id, error = %e, "invalid hostname");
                continue;
            }
        };
        let agent_id = AgentId::new(&beat.agent_id);
        if let Err(e) = agent_uc
            .record_heartbeat(&agent_id, hostname, beat.heartbeat_interval_secs)
            .await
        {
            error!(agent_id = %beat.agent_id, error = %e, "record_heartbeat failed");
        }
    }

    info!("heartbeat listener stream closed");
    Ok(())
}

pub async fn run_shutdown(channel: Channel, agent_uc: Arc<AgentUseCases<PgAgentRepository>>) {
    loop {
        if let Err(e) = shutdown_once(channel.clone(), &agent_uc).await {
            warn!(error = %e, "shutdown listener exited; reconnecting");
        }
        tokio::time::sleep(RECONNECT_BACKOFF).await;
    }
}

async fn shutdown_once(
    channel: Channel,
    agent_uc: &AgentUseCases<PgAgentRepository>,
) -> Result<(), ListenerError> {
    let mut subscriber = Subscriber::new(channel)
        .await
        .map_err(|e| ListenerError::SubscriberInit(e.to_string()))?;
    subscriber
        .subscribe(SHUTDOWN_SUBJECT, None)
        .await
        .map_err(|e| ListenerError::Subscribe {
            subject: SHUTDOWN_SUBJECT.to_string(),
            message: e.to_string(),
        })?;
    info!(subject = SHUTDOWN_SUBJECT, "subscribed");

    while let Some(msg) = subscriber.recv().await {
        let shutdown: AgentShutdown = match serde_json::from_slice(&msg.payload) {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "bad shutdown payload");
                continue;
            }
        };
        let agent_id = AgentId::new(&shutdown.agent_id);
        if let Err(e) = agent_uc.record_shutdown(&agent_id).await {
            warn!(agent_id = %shutdown.agent_id, error = %e, "record_shutdown failed");
        }
    }

    info!("shutdown listener stream closed");
    Ok(())
}
