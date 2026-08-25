use crate::domain::clock;
use crate::domain::ids::AppId;
use chrono::{DateTime, Utc};

/// What an agent reported about the machine it runs on, stored verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentHost {
    pub version: String,
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub cpu_count: Option<i32>,
    pub total_memory_mb: Option<i64>,
    pub reported_at: DateTime<Utc>,
}

/// The 1:1 specialization of an [`App`] that runs jobs. Its identity and
/// credential live on the `App`; this aggregate only marks "this app is a
/// agent" and carries agent-only attributes. `last_seen` is the durable
/// last-activity timestamp (survives a control-plane restart); live
/// online/offline presence is read from the in-memory agent registry, not here.
///
/// [`App`]: crate::domain::app::App
#[derive(Debug, Clone)]
pub struct Agent {
    app_id: AppId,
    last_seen: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    host: Option<AgentHost>,
}

impl Agent {
    #[must_use]
    pub fn from_persistence(
        app_id: AppId,
        last_seen: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        host: Option<AgentHost>,
    ) -> Self {
        Self {
            app_id,
            last_seen,
            created_at,
            host,
        }
    }

    /// A freshly registered agent has never been seen connected yet.
    #[must_use]
    pub fn create(app_id: AppId) -> Self {
        Self {
            app_id,
            last_seen: None,
            created_at: clock::now(),
            host: None,
        }
    }

    #[must_use]
    pub fn app_id(&self) -> &AppId {
        &self.app_id
    }

    #[must_use]
    pub fn last_seen(&self) -> Option<DateTime<Utc>> {
        self.last_seen
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub fn host(&self) -> Option<&AgentHost> {
        self.host.as_ref()
    }
}
