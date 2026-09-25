use crate::application::job::{JobUseCases, ReapOrphanedJobs};
use crate::domain::caller::{CallerContext, ServiceIdentity};
use crate::domain::ids::AppId;
use scylla_extension::Actions;
use std::sync::Arc;
use tracing::{instrument, warn};

/// Level-triggered: compares "running" against the connected set, so restarts and reconnect races need no special path.
pub struct JobReaper {
    actions: Arc<Actions>,
    job_uc: Arc<JobUseCases>,
}

impl JobReaper {
    #[must_use]
    pub fn new(actions: Arc<Actions>, job_uc: Arc<JobUseCases>) -> Self {
        Self { actions, job_uc }
    }

    #[instrument(skip_all, fields(connected = connected.len()))]
    pub async fn reap(&self, connected: &[AppId]) -> u64 {
        let caller = CallerContext::Service(ServiceIdentity::job_reaper());
        let reap = ReapOrphanedJobs {
            connected: connected.to_vec(),
        };
        match self.actions.run(&*self.job_uc, &caller, reap).await {
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
    use crate::test_support::authz::{RecordingPermissionService, actions};
    use crate::test_support::stubs::StubJobs;

    fn reaper(jobs: Arc<StubJobs>) -> JobReaper {
        JobReaper::new(
            Arc::new(actions(Arc::new(RecordingPermissionService::new()))),
            Arc::new(JobUseCases::new(jobs)),
        )
    }

    #[tokio::test]
    async fn reap_forwards_the_connected_set_and_returns_the_count() {
        let jobs = Arc::new(StubJobs::orphaning(3));
        let reaper = reaper(jobs.clone());

        assert_eq!(reaper.reap(&[]).await, 3);
        assert_eq!(reaper.reap(&[AppId::new("agent-1")]).await, 3);

        let swept = jobs.swept();
        assert_eq!(swept.len(), 2);
        assert!(
            swept[0].is_empty(),
            "boot pass reaps with an empty connected set"
        );
        assert_eq!(swept[1], vec![AppId::new("agent-1")]);
    }

    #[tokio::test]
    async fn reap_reports_zero_when_nothing_is_stranded() {
        assert_eq!(reaper(Arc::new(StubJobs::orphaning(0))).reap(&[]).await, 0);
    }
}
