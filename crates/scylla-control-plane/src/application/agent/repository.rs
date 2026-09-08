use crate::domain::agent::{Agent, AgentHost};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, OrganizationId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Aggregate run statistics for a single agent, derived from the `jobs` it
/// executed (counted by status). Computed on read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentStats {
    pub total: i64,
    pub pending: i64,
    pub running: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled: i64,
    pub orphaned: i64,
    pub last_run_at: Option<DateTime<Utc>>,
    /// Finished jobs per day over the last 30 days, oldest first. Days with
    /// no finished job are absent — consumers zero-fill the gaps.
    pub daily: Vec<DailyOutcome>,
    pub median_duration_ms: Option<i64>,
    pub p95_duration_ms: Option<i64>,
}

/// One day of finished-job outcomes for an agent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DailyOutcome {
    pub day: DateTime<Utc>,
    pub completed: i64,
    pub failed: i64,
    pub cancelled: i64,
    pub orphaned: i64,
    pub median_duration_ms: Option<i64>,
}

/// Persistence for the `agents` table — the 1:1 specialization of an app that
/// runs jobs. A row exists only for apps that are agents. Stats are derived
/// from the `jobs` table (`agent_app_id`), not stored here.
#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn find_by_app_id(&self, app_id: &AppId) -> DomainResult<Agent>;
    async fn list_by_organization(&self, org_id: &OrganizationId) -> DomainResult<Vec<Agent>>;
    /// Best-effort upsert of the durable last-activity timestamp. Self-heals a
    /// missing row so presence never depends on the introspection table.
    async fn touch_last_seen(&self, app_id: &AppId, at: DateTime<Utc>) -> DomainResult<()>;
    /// Best-effort upsert of the last reported host. Self-heals a missing row
    /// like `touch_last_seen`.
    async fn record_host(&self, app_id: &AppId, host: &AgentHost) -> DomainResult<()>;
    async fn agent_stats(&self, app_id: &AppId) -> DomainResult<AgentStats>;
}
