use crate::application::agent::dispatch::JobDispatch;
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::domain::entities::AppId;
use crate::domain::errors::{DomainError, DomainResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tracing::warn;

const DISPATCH_QUEUE: usize = 64;

/// In-memory registry of connected agent streams (mono-instance). Maps an App
/// to the sender side of its agent stream; the gRPC handler owns the receiver
/// and forwards dispatches to the wire. Replaces the message broker for job
/// dispatch — presence is simply having an entry here.
///
/// Each registration carries a monotonic **connection id**. Per-connection
/// cleanup ([`unregister_if_current`](Self::unregister_if_current)) only removes
/// the entry when its id still matches — so a slow cleanup of an old, dropped
/// stream can never evict the entry of a *newer* reconnect of the same App
/// (a check-then-act race that an unconditional `remove(app_id)` would hit).
///
/// Lock access recovers from poisoning (`unwrap_or_else(into_inner)`) instead of
/// panicking: the map is plain data with no invariant a panicking thread could
/// corrupt, and a poisoned panic would otherwise take down agent dispatch for
/// the whole instance in a cascade.
/// One live agent connection: its monotonic id, the dispatch channel sender, and
/// the count of jobs dispatched to it but not yet reported terminal (its load).
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

    /// Register a newly-connected agent. Returns the connection id (for
    /// per-connection cleanup) and the receiver the handler forwards to the
    /// client stream. A reconnect replaces the prior sender.
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

    /// Force-remove an App's current stream regardless of connection id. Used by
    /// admin actions (secret revoke / app disable) that must drop whatever stream
    /// is connected right now.
    pub fn unregister(&self, app_id: &AppId) {
        self.agents
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(app_id.as_str());
    }

    /// Remove an App's stream only if `conn_id` is still the live registration.
    /// Used by per-connection cleanup so tearing down a stale connection doesn't
    /// evict a newer reconnect.
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
        // Clone the sender out of the lock (so the await never holds it) and
        // optimistically count the job as in-flight in the same critical section.
        // A failed send rolls the count back via `release`.
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
        // Dropping the stored sender ends the agent's down-stream, which closes
        // the RPC and stops the agent. Removing the entry also clears its
        // in-flight count, so a reconnect starts fresh.
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
