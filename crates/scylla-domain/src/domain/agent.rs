use crate::domain::clock;
use crate::domain::ids::AppId;
use chrono::{DateTime, Utc};

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
