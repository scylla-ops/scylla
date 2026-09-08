use crate::application::job::repository::JobRepository;
use crate::domain::ids::AppId;
use std::sync::Arc;
use tracing::{instrument, warn};

/// Reaps stranded runs. A job is `running` only while its owning agent holds it,
/// so a running job whose agent is not currently connected will never reach a
/// terminal report on its own (the agent crashed, the stream dropped, or the
/// control plane restarted and forgot every live stream). Level-triggered
/// reconciliation — desired state ("running ⇒ a connected agent owns it") vs
/// actual — rather than reacting to disconnect edges, so it is immune to
/// reconnect races and covers a control-plane restart in the same code path.
///
/// Runs once at boot (nothing is connected yet, so every leftover running job
/// from a previous process is orphaned) and on a periodic tick with the live
/// connection set.
pub struct JobReaper<J: JobRepository> {
    job_repo: Arc<J>,
}

impl<J: JobRepository> JobReaper<J> {
    #[must_use]
    pub fn new(job_repo: Arc<J>) -> Self {
        Self { job_repo }
    }

    /// Orphan every `running` job whose agent is not in `connected`. Best-effort:
    /// a failed reconciliation is logged and retried on the next tick, never
    /// propagated. Returns how many jobs were orphaned this pass.
    #[instrument(skip_all, fields(connected = connected.len()))]
    pub async fn reap(&self, connected: &[AppId]) -> u64 {
        match self.job_repo.orphan_running_without_agents(connected).await {
            Ok(0) => 0,
            Ok(n) => {
                warn!(
                    orphaned = n,
                    "reaped running jobs whose agent is no longer connected"
                );
                n
            }
            Err(e) => {
                warn!(error = %e, "job reaper: reconciliation pass failed");
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::pagination::{PaginatedResult, PaginationParams};
    use crate::domain::errors::DomainResult;
    use crate::domain::ids::{JobId, OrganizationId, PipelineId, ProjectId};
    use crate::domain::job::Job;
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// Records the `connected` set each `reap` passed to the repo and returns a
    /// fixed orphan count. Every other method is unused here.
    struct RecordingJobs {
        orphaned: u64,
        seen_connected: Mutex<Vec<Vec<String>>>,
    }

    #[async_trait]
    impl JobRepository for RecordingJobs {
        async fn orphan_running_without_agents(&self, connected: &[AppId]) -> DomainResult<u64> {
            self.seen_connected
                .lock()
                .unwrap()
                .push(connected.iter().map(|a| a.as_str().to_string()).collect());
            Ok(self.orphaned)
        }
        async fn create(&self, _: &Job) -> DomainResult<Job> {
            unimplemented!()
        }
        async fn find_by_id(&self, _: &JobId) -> DomainResult<Job> {
            unimplemented!()
        }
        async fn update(&self, _: &Job) -> DomainResult<Job> {
            unimplemented!()
        }
        async fn set_agent(&self, _: &JobId, _: &AppId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_pending_unassigned(&self) -> DomainResult<Vec<Job>> {
            unimplemented!()
        }
        async fn delete(&self, _: &JobId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_all(
            &self,
            _: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Job>> {
            unimplemented!()
        }
        async fn list_by_pipeline(
            &self,
            _: &PipelineId,
            _: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Job>> {
            unimplemented!()
        }
        async fn list_by_project(
            &self,
            _: &ProjectId,
            _: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Job>> {
            unimplemented!()
        }
        async fn list_by_organization(
            &self,
            _: &OrganizationId,
            _: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Job>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn reap_forwards_the_connected_set_and_returns_the_count() {
        let jobs = Arc::new(RecordingJobs {
            orphaned: 3,
            seen_connected: Mutex::new(vec![]),
        });
        let reaper = JobReaper::new(jobs.clone());

        // Boot pass: nothing connected — the repo is asked to reap unconditionally.
        assert_eq!(reaper.reap(&[]).await, 3);
        // Periodic pass: one live agent is excluded from reaping.
        assert_eq!(reaper.reap(&[AppId::new("agent-1")]).await, 3);

        let seen = jobs.seen_connected.lock().unwrap();
        assert_eq!(seen.len(), 2);
        assert!(
            seen[0].is_empty(),
            "boot pass reaps with an empty connected set"
        );
        assert_eq!(seen[1], vec!["agent-1".to_string()]);
    }

    #[tokio::test]
    async fn reap_reports_zero_when_nothing_is_stranded() {
        let jobs = Arc::new(RecordingJobs {
            orphaned: 0,
            seen_connected: Mutex::new(vec![]),
        });
        let reaper = JobReaper::new(jobs);
        assert_eq!(reaper.reap(&[]).await, 0);
    }
}
