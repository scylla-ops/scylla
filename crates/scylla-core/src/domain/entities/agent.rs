use crate::domain::entities::AgentId;
use crate::domain::value_objects::agent::Hostname;
use chrono::{DateTime, Duration, Utc};
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct Agent {
    id: AgentId,
    hostname: Hostname,
    last_seen_at: DateTime<Utc>,
    shutdown_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Agent {
    #[must_use]
    pub fn create(id: AgentId, hostname: Hostname) -> Self {
        let now = Utc::now();
        Self {
            id,
            hostname,
            last_seen_at: now,
            shutdown_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Refresh presence on heartbeat. Clears any prior graceful-shutdown marker.
    pub fn record_heartbeat(&mut self, hostname: Hostname) {
        let now = Utc::now();
        self.hostname = hostname;
        self.last_seen_at = now;
        self.shutdown_at = None;
        self.updated_at = now;
    }

    /// Record a graceful shutdown. `last_seen_at` stays truthful; derived status
    /// reads `shutdown_at` to report disconnected immediately.
    pub fn record_shutdown(&mut self) {
        let now = Utc::now();
        self.shutdown_at = Some(now);
        self.updated_at = now;
    }

    #[must_use]
    pub fn is_connected(&self, threshold: Duration) -> bool {
        if self.shutdown_at.is_some() {
            return false;
        }
        Utc::now().signed_duration_since(self.last_seen_at) <= threshold
    }

    #[must_use]
    pub fn id(&self) -> &AgentId {
        &self.id
    }

    #[must_use]
    pub fn hostname(&self) -> &Hostname {
        &self.hostname
    }

    #[must_use]
    pub fn last_seen_at(&self) -> DateTime<Utc> {
        self.last_seen_at
    }

    #[must_use]
    pub fn shutdown_at(&self) -> Option<DateTime<Utc>> {
        self.shutdown_at
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
