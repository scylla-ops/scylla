use crate::application::agent::dispatch::assemble_dispatch;
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::application::agent::use_case::{DispatchOutcome, DispatchUseCases};
use crate::application::authz::service::PermissionService;
use crate::application::job::repository::JobRepository;
use crate::application::pipeline::repository::PipelineRepository;
use crate::application::secret::SecretResolver;
use std::sync::Arc;
use tracing::{info, instrument, warn};

/// (Re)dispatches the backlog of jobs that were minted while no agent was
/// connected (or none authorized). A job is persisted by `run_pipeline` before
/// it is handed to an agent, so if dispatch found no worker the job sits
/// `pending` with no `agent_app_id` — without this it would stay there forever.
///
/// `drain` is meant to run when a worker connects (the new agent is idle, so the
/// least-loaded selector hands it the waiting jobs) and as a periodic safety net
/// in case a connect signal is missed.
pub struct PendingJobScheduler<J, P, W, PS>
where
    J: JobRepository,
    P: PipelineRepository,
    W: AgentDispatch,
    PS: PermissionService,
{
    job_repo: Arc<J>,
    pipeline_repo: Arc<P>,
    dispatch_uc: Arc<DispatchUseCases<W, PS>>,
    secret_resolver: Arc<dyn SecretResolver>,
}

impl<J, P, W, PS> PendingJobScheduler<J, P, W, PS>
where
    J: JobRepository,
    P: PipelineRepository,
    W: AgentDispatch,
    PS: PermissionService,
{
    #[must_use]
    pub fn new(
        job_repo: Arc<J>,
        pipeline_repo: Arc<P>,
        dispatch_uc: Arc<DispatchUseCases<W, PS>>,
        secret_resolver: Arc<dyn SecretResolver>,
    ) -> Self {
        Self {
            job_repo,
            pipeline_repo,
            dispatch_uc,
            secret_resolver,
        }
    }

    /// Try to place every pending, unassigned job on a connected eligible agent,
    /// oldest first. Returns how many were dispatched this pass. Best-effort: a
    /// job with no eligible agent is left pending for the next pass, and a single
    /// job's failure never aborts the drain.
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
            // Re-assemble the dispatch via the SAME path the immediate run uses
            // (resolve secrets + overlay the job's persisted inputs), so a job
            // placed here is byte-for-byte what it would have been on dispatch.
            let dispatch =
                match assemble_dispatch(&*self.pipeline_repo, &*self.secret_resolver, &job).await {
                    Ok(dispatch) => dispatch,
                    Err(e) => {
                        warn!(job_id = %job.id(), error = %e, "pending-job drain: dispatch assembly failed; skipping");
                        continue;
                    }
                };
            match self.dispatch_uc.dispatch_job(job.pipeline_id(), &dispatch).await {
                Ok(DispatchOutcome::Dispatched(app_id)) => {
                    if let Err(e) = self.job_repo.set_agent(job.id(), &app_id).await {
                        warn!(job_id = %job.id(), %app_id, error = %e, "pending-job drain: agent attribution failed");
                    }
                    info!(job_id = %job.id(), %app_id, "pending job dispatched to agent");
                    dispatched += 1;
                }
                // Still no eligible agent — leave it pending for a later pass.
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
    use crate::application::agent::dispatch::{DispatchNode, JobDispatch};
    use crate::application::caller::CallerContext;
    use crate::domain::entities::{
        AppId, Job, JobId, OrganizationId, Pipeline, PipelineId, PipelineNode, ProjectId,
    };
    use crate::domain::errors::DomainResult;
    use crate::domain::value_objects::permission::Permission;
    use crate::domain::value_objects::{PaginatedResult, PaginationParams};
    use crate::test_support::organizations::org;
    use crate::test_support::pipelines::pipeline;
    use crate::test_support::projects::project;
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// Job repo holding a fixed pending set; records `set_agent` attributions.
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
        async fn list_all(&self, _: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Job>> {
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

    struct StubPipelines {
        pipeline: Pipeline,
    }

    #[async_trait]
    impl PipelineRepository for StubPipelines {
        async fn find_by_id(&self, _: &PipelineId) -> DomainResult<Pipeline> {
            Ok(self.pipeline.clone())
        }
        async fn create(&self, _: &Pipeline) -> DomainResult<Pipeline> {
            unimplemented!()
        }
        async fn update(&self, _: &Pipeline) -> DomainResult<Pipeline> {
            unimplemented!()
        }
        async fn delete(&self, _: &PipelineId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_all(
            &self,
            _: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Pipeline>> {
            unimplemented!()
        }
        async fn list_by_project(
            &self,
            _: &ProjectId,
            _: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Pipeline>> {
            unimplemented!()
        }
        async fn list_by_organization(
            &self,
            _: &OrganizationId,
            _: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Pipeline>> {
            unimplemented!()
        }
    }

    /// One always-idle connected agent that accepts every dispatch.
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

    struct AllowAll;

    #[async_trait]
    impl PermissionService for AllowAll {
        async fn check(&self, _: &CallerContext, _: Permission) -> DomainResult<()> {
            Ok(())
        }
    }

    /// Resolver stub: maps nodes to dispatch nodes, literal env only (test
    /// pipelines reference no secrets).
    struct StubResolver;

    #[async_trait]
    impl SecretResolver for StubResolver {
        async fn resolve(
            &self,
            _project_id: &ProjectId,
            nodes: &[PipelineNode],
        ) -> DomainResult<Vec<DispatchNode>> {
            Ok(nodes
                .iter()
                .map(|n| DispatchNode {
                    id: n.id().to_string(),
                    deps: n.deps().iter().map(ToString::to_string).collect(),
                    working_dir: n.working_dir().map(|w| w.as_str().to_string()),
                    step: n.step().clone(),
                    env: vec![],
                })
                .collect())
        }
    }

    fn a_pipeline() -> Pipeline {
        pipeline(&project(&org("o"), "p"))
    }

    #[tokio::test]
    async fn drain_dispatches_pending_jobs_and_records_the_agent() {
        let pl = a_pipeline();
        let job = crate::test_support::jobs::job(&pl); // Pending, unassigned
        let job_id = job.id().to_string();

        let jobs = Arc::new(StubJobs {
            pending: vec![job],
            assigned: Mutex::new(vec![]),
        });
        let registry = Arc::new(StubRegistry {
            dispatched: Mutex::new(vec![]),
        });
        let dispatch_uc = Arc::new(DispatchUseCases::new(registry.clone(), Arc::new(AllowAll)));
        let scheduler = PendingJobScheduler::new(
            jobs.clone(),
            Arc::new(StubPipelines { pipeline: pl }),
            dispatch_uc,
            Arc::new(StubResolver),
        );

        assert_eq!(scheduler.drain().await, 1, "the one pending job is dispatched");
        assert_eq!(registry.dispatched.lock().unwrap().as_slice(), ["agent-1"]);
        let assigned = jobs.assigned.lock().unwrap();
        assert_eq!(assigned.as_slice(), [(job_id, "agent-1".to_string())]);
    }

    #[tokio::test]
    async fn drain_is_noop_with_no_pending_jobs() {
        let registry = Arc::new(StubRegistry {
            dispatched: Mutex::new(vec![]),
        });
        let dispatch_uc = Arc::new(DispatchUseCases::new(registry.clone(), Arc::new(AllowAll)));
        let scheduler = PendingJobScheduler::new(
            Arc::new(StubJobs {
                pending: vec![],
                assigned: Mutex::new(vec![]),
            }),
            Arc::new(StubPipelines {
                pipeline: a_pipeline(),
            }),
            dispatch_uc,
            Arc::new(StubResolver),
        );

        assert_eq!(scheduler.drain().await, 0);
        assert!(registry.dispatched.lock().unwrap().is_empty());
    }
}
