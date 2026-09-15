use crate::application::agent::dispatch::JobDispatch;
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::AppId;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tracing::warn;

const DISPATCH_QUEUE: usize = 64;

/// Cleanup is generation-checked: a slow teardown of an old stream must never evict a newer reconnect.
/// Lock poisoning is recovered: the map is plain data, and a panic here would take down dispatch for the instance.
struct Conn {
    conn_id: u64,
    tx: mpsc::Sender<JobDispatch>,
    in_flight: u32,
}

#[derive(Default)]
pub struct InMemoryAgentRegistry {
    agents: Mutex<HashMap<String, Conn>>,
    next_conn: AtomicU64,
}

impl InMemoryAgentRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, app_id: &AppId) -> (u64, mpsc::Receiver<JobDispatch>) {
        let (tx, rx) = mpsc::channel(DISPATCH_QUEUE);
        let conn_id = self.next_conn.fetch_add(1, Ordering::Relaxed);
        let mut map = self
            .agents
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if map
            .insert(
                app_id.as_str().to_string(),
                Conn {
                    conn_id,
                    tx,
                    in_flight: 0,
                },
            )
            .is_some()
        {
            warn!(app_id = %app_id, "agent reconnected; replacing previous stream");
        }
        (conn_id, rx)
    }

    pub fn unregister(&self, app_id: &AppId) {
        self.agents
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(app_id.as_str());
    }

    pub fn unregister_if_current(&self, app_id: &AppId, conn_id: u64) {
        let mut map = self
            .agents
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if map
            .get(app_id.as_str())
            .is_some_and(|c| c.conn_id == conn_id)
        {
            map.remove(app_id.as_str());
        }
    }
}

#[async_trait]
impl AgentDispatch for InMemoryAgentRegistry {
    fn connected(&self) -> Vec<AppId> {
        self.agents
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .map(AppId::new)
            .collect()
    }

    async fn dispatch(&self, app_id: &AppId, dispatch: &JobDispatch) -> DomainResult<()> {
        // Clone the sender out of the lock so the await never holds it; a failed send rolls the count back.
        let sender = {
            let mut map = self
                .agents
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            map.get_mut(app_id.as_str()).map(|c| {
                c.in_flight += 1;
                c.tx.clone()
            })
        };
        match sender {
            Some(tx) => tx.send(dispatch.clone()).await.map_err(|_| {
                self.release(app_id);
                DomainError::infrastructure(format!("agent {app_id} stream closed"))
            }),
            None => Err(DomainError::infrastructure(format!(
                "agent {app_id} not connected"
            ))),
        }
    }

    fn disconnect(&self, app_id: &AppId) {
        self.unregister(app_id);
    }

    fn in_flight(&self, app_id: &AppId) -> usize {
        self.agents
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(app_id.as_str())
            .map_or(0, |c| c.in_flight as usize)
    }

    fn release(&self, app_id: &AppId) {
        if let Some(c) = self
            .agents
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_mut(app_id.as_str())
        {
            c.in_flight = c.in_flight.saturating_sub(1);
        }
    }
}
