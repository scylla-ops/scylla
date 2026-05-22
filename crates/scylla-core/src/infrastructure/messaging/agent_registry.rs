use crate::application::agent::dispatch_port::AgentDispatch;
use crate::domain::entities::AppId;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::pipeline::JobDispatch;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tracing::warn;

const DISPATCH_QUEUE: usize = 64;

/// In-memory registry of connected agent streams (mono-instance). Maps an App
/// to the sender side of its agent stream; the gRPC handler owns the receiver
/// and forwards dispatches to the wire. Replaces the message broker for job
/// dispatch — presence is simply having an entry here.
#[derive(Default)]
pub struct InMemoryAgentRegistry {
    agents: Mutex<HashMap<String, mpsc::Sender<JobDispatch>>>,
}

impl InMemoryAgentRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a newly-connected agent and return the receiver the handler
    /// forwards to the client stream. A reconnect replaces the prior sender.
    pub fn register(&self, app_id: &AppId) -> mpsc::Receiver<JobDispatch> {
        let (tx, rx) = mpsc::channel(DISPATCH_QUEUE);
        let mut map = self.agents.lock().expect("agent registry lock poisoned");
        if map.insert(app_id.as_str().to_string(), tx).is_some() {
            warn!(app_id = %app_id, "agent reconnected; replacing previous stream");
        }
        rx
    }

    /// Remove a agent on disconnect.
    pub fn unregister(&self, app_id: &AppId) {
        self.agents
            .lock()
            .expect("agent registry lock poisoned")
            .remove(app_id.as_str());
    }
}

#[async_trait]
impl AgentDispatch for InMemoryAgentRegistry {
    fn connected(&self) -> Vec<AppId> {
        self.agents
            .lock()
            .expect("agent registry lock poisoned")
            .keys()
            .map(AppId::new)
            .collect()
    }

    async fn dispatch(&self, app_id: &AppId, dispatch: &JobDispatch) -> DomainResult<()> {
        // Clone the sender out of the lock so the await never holds it.
        let sender = self
            .agents
            .lock()
            .expect("agent registry lock poisoned")
            .get(app_id.as_str())
            .cloned();
        match sender {
            Some(tx) => tx
                .send(dispatch.clone())
                .await
                .map_err(|_| DomainError::infrastructure(format!("agent {app_id} stream closed"))),
            None => Err(DomainError::infrastructure(format!(
                "agent {app_id} not connected"
            ))),
        }
    }

    fn disconnect(&self, app_id: &AppId) {
        // Dropping the stored sender ends the agent's down-stream, which closes
        // the RPC and stops the agent.
        self.unregister(app_id);
    }
}
