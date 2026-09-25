use crate::application::agent::dispatch_use_case::{DispatchPendingJobs, DispatchUseCases};
use crate::domain::caller::{CallerContext, ServiceIdentity};
use scylla_extension::Actions;
use std::sync::Arc;
use tracing::{info, instrument, warn};

/// A job persisted while no eligible agent was connected stays `pending` with no agent; this places it later.
pub struct PendingJobScheduler {
    actions: Arc<Actions>,
    dispatch_uc: Arc<DispatchUseCases>,
}

impl PendingJobScheduler {
    #[must_use]
    pub fn new(actions: Arc<Actions>, dispatch_uc: Arc<DispatchUseCases>) -> Self {
        Self {
            actions,
            dispatch_uc,
        }
    }

    #[instrument(skip(self))]
    pub async fn drain(&self) -> usize {
        let caller = CallerContext::Service(ServiceIdentity::job_dispatcher());
        match self
            .actions
            .run(&*self.dispatch_uc, &caller, DispatchPendingJobs)
            .await
        {
            Ok(placed) if placed.is_empty() => 0,
            Ok(placed) => {
                info!(
                    dispatched = placed.len(),
                    "pending-job drain placed waiting jobs"
                );
                placed.len()
            }
            Err(e) => {
                warn!(error = %e, "pending-job drain: could not list pending jobs");
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::AppId;
    use crate::domain::pipeline::Pipeline;
    use crate::test_support::authz::{RecordingPermissionService, actions};
    use crate::test_support::jobs::job;
    use crate::test_support::organizations::org;
    use crate::test_support::pipelines::pipeline;
    use crate::test_support::projects::project;
    use crate::test_support::stubs::{EchoResolver, OnePipeline, StubJobs, StubRegistry};

    fn scheduler(
        pl: Pipeline,
        jobs: Arc<StubJobs>,
        registry: Arc<StubRegistry>,
    ) -> PendingJobScheduler {
        let permissions = Arc::new(RecordingPermissionService::new());
        let dispatch_uc = Arc::new(DispatchUseCases::new(
            registry,
            permissions.clone(),
            jobs,
            Arc::new(OnePipeline(pl)),
            Arc::new(EchoResolver),
        ));
        PendingJobScheduler::new(Arc::new(actions(permissions)), dispatch_uc)
    }

    #[tokio::test]
    async fn drain_dispatches_pending_jobs_and_records_the_agent() {
        let pl = pipeline(&project(&org("o"), "p"));
        let pending = job(&pl);
        let job_id = pending.id().clone();
        let jobs = Arc::new(StubJobs::with(vec![pending]));
        let registry = Arc::new(StubRegistry::accepting());
        registry.connect(&AppId::new("agent-1"));

        assert_eq!(
            scheduler(pl, jobs.clone(), registry.clone()).drain().await,
            1,
            "the one pending job is dispatched"
        );
        assert_eq!(registry.dispatched_to(), vec![AppId::new("agent-1")]);
        assert_eq!(jobs.assigned(), vec![(job_id, AppId::new("agent-1"))]);
    }

    #[tokio::test]
    async fn drain_is_noop_with_no_pending_jobs() {
        let registry = Arc::new(StubRegistry::default());
        registry.connect(&AppId::new("agent-1"));

        assert_eq!(
            scheduler(
                pipeline(&project(&org("o"), "p")),
                Arc::new(StubJobs::default()),
                registry.clone()
            )
            .drain()
            .await,
            0
        );
        assert!(registry.dispatched().is_empty());
    }
}
