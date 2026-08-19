use crate::domain::agent::Agent;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, OrganizationId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Aggregate run statistics for a single agent, derived from the `jobs` it
/// executed (counted by status). Not stored — computed on read. The status
/// counters partition `total`: every `JobStatus` variant has a bucket, so a
/// consumer summing them gets `total` back.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentStats {
    pub total: i64,
    pub pending: i64,
    pub running: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled: i64,
    /// Jobs the reaper stamped after their agent vanished mid-run.
    pub orphaned: i64,
    pub last_run_at: Option<DateTime<Utc>>,
    /// Finished jobs per day over the last 30 days, oldest first. Days with
    /// no finished job are absent — consumers zero-fill the gaps.
    pub daily: Vec<DailyOutcome>,
    /// Wall-clock run duration over all jobs that both started and finished.
    /// `None` when the agent has none yet — distinct from a 0 ms run.
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
    /// Median run duration that day; `None` if no job that day has both
    /// timestamps (e.g. every one was skipped or cancelled before starting).
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
    async fn agent_stats(&self, app_id: &AppId) -> DomainResult<AgentStats>;
}
