use crate::application::agent::dispatch::assemble_dispatch;
use crate::application::agent::dispatch_use_case::{DispatchOutcome, DispatchUseCases};
use crate::application::job::repository::JobRepository;
use crate::application::pipeline::repository::PipelineRepository;
use crate::application::secret::SecretResolver;
use std::sync::Arc;
use tracing::{info, instrument, warn};

/// A job persisted while no eligible agent was connected stays `pending` with no agent; this places it later.
pub struct PendingJobScheduler {
    job_repo: Arc<dyn JobRepository>,
    pipeline_repo: Arc<dyn PipelineRepository>,
    dispatch_uc: Arc<DispatchUseCases>,
    secret_resolver: Arc<dyn SecretResolver>,
}

impl PendingJobScheduler {
    #[must_use]
    pub fn new(
        job_repo: Arc<dyn JobRepository>,
        pipeline_repo: Arc<dyn PipelineRepository>,
        dispatch_uc: Arc<DispatchUseCases>,
        secret_resolver: Arc<dyn SecretResolver>,
    ) -> Self {
        Self {
            job_repo,
            pipeline_repo,
            dispatch_uc,
            secret_resolver,
        }
    }

    #[instrument(skip(self))]
    pub async fn drain(&self) -> usize {
        let jobs = match self.job_repo.list_pending_unassigned().await {
            Ok(jobs) => jobs,
            Err(e) => {
                warn!(error = %e, "pending-job drain: could not list pending jobs");
                return 0;
            }
        };
        if jobs.is_empty() {
            return 0;
        }

        let mut dispatched = 0usize;
        for job in jobs {
            let dispatch = match assemble_dispatch(
                &*self.pipeline_repo,
                &*self.secret_resolver,
                &job,
            )
            .await
            {
                Ok(dispatch) => dispatch,
                Err(e) => {
                    warn!(job_id = %job.id(), error = %e, "pending-job drain: dispatch assembly failed; skipping");
                    continue;
                }
            };
            match self
                .dispatch_uc
                .dispatch_job(job.pipeline_id(), &dispatch)
                .await
            {
                Ok(DispatchOutcome::Dispatched(app_id)) => {
                    if let Err(e) = self.job_repo.set_agent(job.id(), &app_id).await {
                        warn!(job_id = %job.id(), %app_id, error = %e, "pending-job drain: agent attribution failed");
                    }
                    info!(job_id = %job.id(), %app_id, "pending job dispatched to agent");
                    dispatched += 1;
                }
                Ok(DispatchOutcome::NoAgentAvailable) => {}
                Err(e) => {
                    warn!(job_id = %job.id(), error = %e, "pending-job drain: dispatch failed");
                }
            }
        }
        if dispatched > 0 {
            info!(dispatched, "pending-job drain placed waiting jobs");
        }
        dispatched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::agent::dispatch::JobDispatch;
    use crate::application::agent::dispatch_port::AgentDispatch;
    use crate::application::pagination::{PaginatedResult, PaginationParams};
    use crate::domain::errors::DomainResult;
    use crate::domain::ids::{AppId, JobId, OrganizationId, PipelineId, ProjectId};
    use crate::domain::job::Job;
    use crate::domain::pipeline::Pipeline;
    use crate::test_support::authz::RecordingPermissionService;
    use crate::test_support::organizations::org;
    use crate::test_support::pipelines::pipeline;
    use crate::test_support::projects::project;
    use crate::test_support::stubs::{EchoResolver, OnePipeline};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct StubJobs {
        pending: Vec<Job>,
        assigned: Mutex<Vec<(String, String)>>,
    }

    #[async_trait]
    impl JobRepository for StubJobs {
        async fn list_pending_unassigned(&self) -> DomainResult<Vec<Job>> {
            Ok(self.pending.clone())
        }
        async fn set_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()> {
            self.assigned
                .lock()
                .unwrap()
                .push((job_id.to_string(), app_id.as_str().to_string()));
            Ok(())
        }
        async fn orphan_running_without_agents(&self, _: &[AppId]) -> DomainResult<u64> {
            unimplemented!()
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

    struct StubRegistry {
        dispatched: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl AgentDispatch for StubRegistry {
        fn connected(&self) -> Vec<AppId> {
            vec![AppId::new("agent-1")]
        }
        async fn dispatch(&self, app_id: &AppId, _: &JobDispatch) -> DomainResult<()> {
            self.dispatched
                .lock()
                .unwrap()
                .push(app_id.as_str().to_string());
            Ok(())
        }
        fn disconnect(&self, _: &AppId) {}
        fn in_flight(&self, _: &AppId) -> usize {
            0
        }
        fn release(&self, _: &AppId) {}
    }

    fn a_pipeline() -> Pipeline {
        pipeline(&project(&org("o"), "p"))
    }

    #[tokio::test]
    async fn drain_dispatches_pending_jobs_and_records_the_agent() {
        let pl = a_pipeline();
        let job = crate::test_support::jobs::job(&pl);
        let job_id = job.id().to_string();

        let jobs = Arc::new(StubJobs {
            pending: vec![job],
            assigned: Mutex::new(vec![]),
        });
        let registry = Arc::new(StubRegistry {
            dispatched: Mutex::new(vec![]),
        });
        let dispatch_uc = Arc::new(DispatchUseCases::new(
            registry.clone(),
            Arc::new(RecordingPermissionService::new()),
        ));
        let scheduler = PendingJobScheduler::new(
            jobs.clone(),
            Arc::new(OnePipeline(pl)),
            dispatch_uc,
            Arc::new(EchoResolver),
        );

        assert_eq!(
            scheduler.drain().await,
            1,
            "the one pending job is dispatched"
        );
        assert_eq!(registry.dispatched.lock().unwrap().as_slice(), ["agent-1"]);
        let assigned = jobs.assigned.lock().unwrap();
        assert_eq!(assigned.as_slice(), [(job_id, "agent-1".to_string())]);
    }

    #[tokio::test]
    async fn drain_is_noop_with_no_pending_jobs() {
        let registry = Arc::new(StubRegistry {
            dispatched: Mutex::new(vec![]),
        });
        let dispatch_uc = Arc::new(DispatchUseCases::new(
            registry.clone(),
            Arc::new(RecordingPermissionService::new()),
        ));
        let scheduler = PendingJobScheduler::new(
            Arc::new(StubJobs {
                pending: vec![],
                assigned: Mutex::new(vec![]),
            }),
            Arc::new(OnePipeline(a_pipeline())),
            dispatch_uc,
            Arc::new(EchoResolver),
        );

        assert_eq!(scheduler.drain().await, 0);
        assert!(registry.dispatched.lock().unwrap().is_empty());
    }
}
