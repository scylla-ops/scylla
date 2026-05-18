use crate::domain::entities::AgentId;
use crate::domain::value_objects::agent::Hostname;
use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub struct Agent {
    id: AgentId,
    hostname: Hostname,
    last_seen_at: DateTime<Utc>,
    shutdown_at: Option<DateTime<Utc>>,
    heartbeat_interval_secs: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Number of missed heartbeats tolerated before an agent is considered stale.
/// Threshold = `heartbeat_interval_secs * MISSED_HEARTBEAT_GRACE`.
const MISSED_HEARTBEAT_GRACE: i64 = 3;

impl Agent {
    #[must_use]
    pub fn from_persistence(
        id: AgentId,
        hostname: Hostname,
        last_seen_at: DateTime<Utc>,
        shutdown_at: Option<DateTime<Utc>>,
        heartbeat_interval_secs: u64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            hostname,
            last_seen_at,
            shutdown_at,
            heartbeat_interval_secs,
            created_at,
            updated_at,
        }
    }

    #[must_use]
    pub fn create(id: AgentId, hostname: Hostname, heartbeat_interval_secs: u64) -> Self {
        let now = Utc::now();
        Self {
            id,
            hostname,
            last_seen_at: now,
            shutdown_at: None,
            heartbeat_interval_secs,
            created_at: now,
            updated_at: now,
        }
    }

    /// Refresh presence on heartbeat. Clears any prior graceful-shutdown marker
    /// and refreshes the agent's self-reported heartbeat interval.
    pub fn record_heartbeat(&mut self, hostname: Hostname, heartbeat_interval_secs: u64) {
        let now = Utc::now();
        self.hostname = hostname;
        self.last_seen_at = now;
        self.shutdown_at = None;
        self.heartbeat_interval_secs = heartbeat_interval_secs;
        self.updated_at = now;
    }

    /// Record a graceful shutdown. `last_seen_at` stays truthful; derived status
    /// reads `shutdown_at` to report disconnected immediately.
    pub fn record_shutdown(&mut self) {
        let now = Utc::now();
        self.shutdown_at = Some(now);
        self.updated_at = now;
    }

    /// Liveness check using the agent's self-reported heartbeat interval.
    /// Considered connected if `now - last_seen_at <= interval * grace`.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        if self.shutdown_at.is_some() {
            return false;
        }
        let secs = i64::try_from(self.heartbeat_interval_secs)
            .unwrap_or(i64::MAX)
            .saturating_mul(MISSED_HEARTBEAT_GRACE);
        let threshold = Duration::try_seconds(secs).unwrap_or(Duration::MAX);
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
    pub fn heartbeat_interval_secs(&self) -> u64 {
        self.heartbeat_interval_secs
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent(interval_secs: u64) -> Agent {
        Agent::create(
            AgentId::new("agent-1"),
            Hostname::new("host-1").unwrap(),
            interval_secs,
        )
    }

    #[test]
    fn fresh_agent_is_connected() {
        let a = make_agent(5);
        assert!(a.is_connected());
        assert!(a.shutdown_at().is_none());
    }

    #[test]
    fn shutdown_forces_disconnect() {
        let mut a = make_agent(5);
        a.record_shutdown();
        assert!(!a.is_connected());
        assert!(a.shutdown_at().is_some());
    }

    #[test]
    fn heartbeat_clears_shutdown() {
        let mut a = make_agent(5);
        a.record_shutdown();
        a.record_heartbeat(Hostname::new("host-2").unwrap(), 10);
        assert!(a.shutdown_at().is_none());
        assert!(a.is_connected());
        assert_eq!(a.hostname().as_str(), "host-2");
        assert_eq!(a.heartbeat_interval_secs(), 10);
    }

    #[test]
    fn disconnected_when_last_seen_outside_grace() {
        let mut a = make_agent(5);
        // Push last_seen_at well beyond grace window (5 * 3 = 15s).
        a.last_seen_at = Utc::now() - Duration::seconds(60);
        assert!(!a.is_connected());
    }

    #[test]
    fn connected_when_last_seen_inside_grace() {
        let mut a = make_agent(5);
        a.last_seen_at = Utc::now() - Duration::seconds(10);
        assert!(a.is_connected());
    }

    #[test]
    fn overflow_interval_does_not_panic() {
        let mut a = make_agent(u64::MAX);
        a.last_seen_at = Utc::now() - Duration::seconds(1);
        assert!(a.is_connected());
    }
}
