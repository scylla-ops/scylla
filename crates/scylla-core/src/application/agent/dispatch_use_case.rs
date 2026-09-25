//! The placement of a stored job on a connected agent. `place` runs in the `commit` closure of
//! `RunPipeline`, `RunPipelineWithInputs` and `DispatchPendingJobs`, after the job row exists.

use crate::application::actions::service_only;
use crate::application::agent::dispatch::{JobDispatch, assemble_dispatch};
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::application::{JobRepository, PipelineRepository, SecretResolver};
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, PipelineId};
use crate::domain::job::Job;
use crate::domain::permission::Permission;
use async_trait::async_trait;
use scylla_auth::authz::PermissionService;
use scylla_extension::{
    Access, Authorized, Command, Committed, Describe, Persist, Prepare, Prepared, Run,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{info, warn};

pub enum DispatchOutcome {
    Dispatched(AppId),
    NoAgentAvailable,
}

/// The stage runner of `DispatchPendingJobs`. `permission_service` asks for `ExecuteJob` only to
/// choose an agent, never to refuse.
pub struct DispatchUseCases {
    registry: Arc<dyn AgentDispatch>,
    permission_service: Arc<dyn PermissionService>,
    job_repo: Arc<dyn JobRepository>,
    pipeline_repo: Arc<dyn PipelineRepository>,
    secret_resolver: Arc<dyn SecretResolver>,
    next: AtomicUsize,
}

impl DispatchUseCases {
    #[must_use]
    pub fn new(
        registry: Arc<dyn AgentDispatch>,
        permission_service: Arc<dyn PermissionService>,
        job_repo: Arc<dyn JobRepository>,
        pipeline_repo: Arc<dyn PipelineRepository>,
        secret_resolver: Arc<dyn SecretResolver>,
    ) -> Self {
        Self {
            registry,
            permission_service,
            job_repo,
            pipeline_repo,
            secret_resolver,
            next: AtomicUsize::new(0),
        }
    }

    /// Best-effort once the job exists: a failed attribution is logged, and the job runs anyway.
    pub(crate) async fn place(&self, job: &mut Job, dispatch: &JobDispatch) -> DispatchOutcome {
        let outcome = self.dispatch_job(job.pipeline_id(), dispatch).await;
        if let DispatchOutcome::Dispatched(app_id) = &outcome {
            info!(job_id = %job.id(), %app_id, "job dispatched to agent");
            match self.job_repo.set_agent(job.id(), app_id).await {
                Ok(()) => job.assign_agent(app_id.clone()),
                Err(e) => {
                    warn!(job_id = %job.id(), %app_id, error = %e, "failed to record job agent attribution");
                }
            }
        }
        outcome
    }

    async fn dispatch_job(
        &self,
        pipeline_id: &PipelineId,
        dispatch: &JobDispatch,
    ) -> DispatchOutcome {
        let agents = self.registry.connected();
        if agents.is_empty() {
            warn!(pipeline_id = %pipeline_id, "no connected agent; job left pending");
            return DispatchOutcome::NoAgentAvailable;
        }

        // Idlest first, then rotate among equals so eligible agents take turns.
        let start = self.next.fetch_add(1, Ordering::Relaxed);
        let n = agents.len();
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by_key(|&i| {
            (
                self.registry.in_flight(&agents[i]),
                start.wrapping_add(i) % n,
            )
        });
        for i in order {
            let app_id = &agents[i];
            let caller = CallerContext::App(app_id.clone());
            match self
                .permission_service
                .check(&caller, Permission::ExecuteJob(pipeline_id.clone()))
                .await
            {
                Ok(()) => match self.registry.dispatch(app_id, dispatch).await {
                    Ok(()) => return DispatchOutcome::Dispatched(app_id.clone()),
                    // Disconnected since `connected()` was snapshotted: try the next one.
                    Err(e) => {
                        warn!(app_id = %app_id, error = %e, "dispatch to agent failed; trying next");
                    }
                },
                Err(DomainError::Forbidden(_)) => {}
                Err(e) => {
                    warn!(app_id = %app_id, error = %e, "authz check errored during dispatch; skipping agent");
                }
            }
        }
        warn!(
            pipeline_id = %pipeline_id,
            "no connected agent authorized to execute pipeline; job left pending"
        );
        DispatchOutcome::NoAgentAvailable
    }
}

/// One pass over the jobs stored while no eligible agent was connected. A job whose dispatch
/// cannot be assembled is logged and skipped; `Committed` is the jobs an agent took.
#[derive(Debug)]
pub struct DispatchPendingJobs;

impl Describe for DispatchPendingJobs {
    fn access(&self) -> Access {
        Access::Authenticated
    }
}

impl Command for DispatchPendingJobs {
    type Staged = Vec<(Job, JobDispatch)>;
    type Committed = Vec<Job>;
}

#[async_trait]
impl Run<Prepare<DispatchPendingJobs>> for DispatchUseCases {
    async fn run(
        &self,
        input: Authorized<DispatchPendingJobs>,
    ) -> DomainResult<Prepared<DispatchPendingJobs>> {
        service_only(input.caller())?;
        let jobs = self.job_repo.list_pending_unassigned().await?;
        let mut staged = Vec::with_capacity(jobs.len());
        for job in jobs {
            match assemble_dispatch(&*self.pipeline_repo, &*self.secret_resolver, &job).await {
                Ok(dispatch) => staged.push((job, dispatch)),
                Err(e) => {
                    warn!(job_id = %job.id(), error = %e, "pending-job drain: dispatch assembly failed; skipping");
                }
            }
        }
        Ok(input.prepared(staged))
    }
}

#[async_trait]
impl Run<Persist<DispatchPendingJobs>> for DispatchUseCases {
    async fn run(
        &self,
        input: Prepared<DispatchPendingJobs>,
    ) -> DomainResult<Committed<DispatchPendingJobs>> {
        input
            .commit(async |staged| {
                let mut placed = Vec::new();
                for (mut job, dispatch) in staged {
                    if let DispatchOutcome::Dispatched(_) = self.place(&mut job, &dispatch).await {
                        placed.push(job);
                    }
                }
                Ok(placed)
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::caller::ServiceIdentity;
    use crate::domain::pipeline::Pipeline;
    use crate::test_support::authz::{RecordingPermissionService, actions};
    use crate::test_support::jobs::job;
    use crate::test_support::organizations::org;
    use crate::test_support::pipelines::pipeline;
    use crate::test_support::projects::project;
    use crate::test_support::stubs::{EchoResolver, OnePipeline, StubJobs, alice};
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct StubRegistry {
        connected: Vec<AppId>,
        dispatched: Mutex<Vec<String>>,
        loads: Mutex<HashMap<String, usize>>,
    }

    impl StubRegistry {
        fn new(connected: Vec<AppId>) -> Self {
            Self {
                connected,
                dispatched: Mutex::new(vec![]),
                loads: Mutex::new(HashMap::new()),
            }
        }
        fn with_loads(connected: Vec<AppId>, loads: HashMap<String, usize>) -> Self {
            Self {
                connected,
                dispatched: Mutex::new(vec![]),
                loads: Mutex::new(loads),
            }
        }
    }

    #[async_trait]
    impl AgentDispatch for StubRegistry {
        fn connected(&self) -> Vec<AppId> {
            self.connected.clone()
        }
        async fn dispatch(&self, app_id: &AppId, _dispatch: &JobDispatch) -> DomainResult<()> {
            self.dispatched
                .lock()
                .unwrap()
                .push(app_id.as_str().to_string());
            *self
                .loads
                .lock()
                .unwrap()
                .entry(app_id.as_str().to_string())
                .or_insert(0) += 1;
            Ok(())
        }
        fn disconnect(&self, _app_id: &AppId) {}
        fn in_flight(&self, app_id: &AppId) -> usize {
            *self
                .loads
                .lock()
                .unwrap()
                .get(app_id.as_str())
                .unwrap_or(&0)
        }
        fn release(&self, app_id: &AppId) {
            if let Some(v) = self.loads.lock().unwrap().get_mut(app_id.as_str()) {
                *v = v.saturating_sub(1);
            }
        }
    }

    struct StubPerms {
        allowed: &'static str,
    }

    #[async_trait]
    impl PermissionService for StubPerms {
        async fn check(&self, caller: &CallerContext, _perm: Permission) -> DomainResult<()> {
            if matches!(caller, CallerContext::App(id) if id.as_str() == self.allowed) {
                Ok(())
            } else {
                Err(DomainError::forbidden("not the allowed agent"))
            }
        }
    }

    fn a_pipeline() -> Pipeline {
        pipeline(&project(&org("o"), "p"))
    }

    fn uc(
        registry: Arc<StubRegistry>,
        permissions: Arc<dyn PermissionService>,
        jobs: Arc<StubJobs>,
        pl: Pipeline,
    ) -> DispatchUseCases {
        DispatchUseCases::new(
            registry,
            permissions,
            jobs,
            Arc::new(OnePipeline(pl)),
            Arc::new(EchoResolver),
        )
    }

    fn dispatch() -> JobDispatch {
        JobDispatch {
            job_id: "j1".to_string(),
            pipeline_id: "pl1".to_string(),
            nodes: vec![],
        }
    }

    #[tokio::test]
    async fn dispatches_to_first_authorized_connected_agent() {
        let registry = Arc::new(StubRegistry::new(vec![
            AppId::new("app-unauthorized"),
            AppId::new("app-ok"),
        ]));
        let uc = uc(
            registry.clone(),
            Arc::new(StubPerms { allowed: "app-ok" }),
            Arc::default(),
            a_pipeline(),
        );

        let outcome = uc.dispatch_job(&PipelineId::new("pl1"), &dispatch()).await;

        assert!(matches!(outcome, DispatchOutcome::Dispatched(id) if id.as_str() == "app-ok"));
        assert_eq!(registry.dispatched.lock().unwrap().as_slice(), ["app-ok"]);
    }

    #[tokio::test]
    async fn spreads_jobs_round_robin_across_authorized_agents() {
        let registry = Arc::new(StubRegistry::new(vec![
            AppId::new("app-a"),
            AppId::new("app-b"),
        ]));
        let uc = uc(
            registry.clone(),
            Arc::new(RecordingPermissionService::new()),
            Arc::default(),
            a_pipeline(),
        );

        for _ in 0..4 {
            uc.dispatch_job(&PipelineId::new("pl1"), &dispatch()).await;
        }

        let dispatched = registry.dispatched.lock().unwrap().clone();
        assert_eq!(dispatched.len(), 4);
        assert_eq!(
            dispatched.iter().filter(|x| x.as_str() == "app-a").count(),
            2,
            "app-a should get half the jobs"
        );
        assert_eq!(
            dispatched.iter().filter(|x| x.as_str() == "app-b").count(),
            2,
            "app-b should get half the jobs (no longer starved)"
        );
    }

    #[tokio::test]
    async fn picks_least_loaded_eligible_agent() {
        let mut loads = HashMap::new();
        loads.insert("app-busy".to_string(), 3);
        let registry = Arc::new(StubRegistry::with_loads(
            vec![AppId::new("app-busy"), AppId::new("app-idle")],
            loads,
        ));
        let uc = uc(
            registry.clone(),
            Arc::new(RecordingPermissionService::new()),
            Arc::default(),
            a_pipeline(),
        );

        uc.dispatch_job(&PipelineId::new("pl1"), &dispatch()).await;

        assert_eq!(registry.dispatched.lock().unwrap().as_slice(), ["app-idle"]);
    }

    #[tokio::test]
    async fn no_agent_when_none_authorized() {
        let registry = Arc::new(StubRegistry::new(vec![AppId::new("app-x")]));
        let uc = uc(
            registry,
            Arc::new(StubPerms { allowed: "nobody" }),
            Arc::default(),
            a_pipeline(),
        );

        let outcome = uc.dispatch_job(&PipelineId::new("pl1"), &dispatch()).await;

        assert!(matches!(outcome, DispatchOutcome::NoAgentAvailable));
    }

    #[tokio::test]
    async fn a_pending_pass_places_each_job_skips_an_unassemblable_one_and_asks_no_permission() {
        let pl = a_pipeline();
        let placeable = job(&pl);
        let orphan = job(&a_pipeline());
        let jobs = Arc::new(StubJobs::with(vec![placeable.clone(), orphan]));
        let registry = Arc::new(StubRegistry::new(vec![AppId::new("agent-1")]));
        let permissions = Arc::new(RecordingPermissionService::new());
        let uc = uc(registry.clone(), permissions.clone(), jobs.clone(), pl);
        let caller = CallerContext::Service(ServiceIdentity::job_dispatcher());

        let placed = actions(permissions.clone())
            .run(&uc, &caller, DispatchPendingJobs)
            .await
            .unwrap();

        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].agent_app_id(), Some(&AppId::new("agent-1")));
        assert_eq!(
            jobs.assigned(),
            vec![(placeable.id().clone(), AppId::new("agent-1"))]
        );
        assert_eq!(
            permissions.checks(),
            vec![(
                CallerContext::App(AppId::new("agent-1")),
                Permission::ExecuteJob(placeable.pipeline_id().clone())
            )],
            "the only check chooses the agent"
        );
    }

    #[tokio::test]
    async fn a_pending_pass_refuses_a_caller_that_is_not_a_service() {
        let pl = a_pipeline();
        let jobs = Arc::new(StubJobs::with(vec![job(&pl)]));
        let registry = Arc::new(StubRegistry::new(vec![AppId::new("agent-1")]));
        let permissions = Arc::new(RecordingPermissionService::new());
        let uc = uc(registry.clone(), permissions.clone(), jobs.clone(), pl);

        let err = actions(permissions)
            .run(&uc, &alice(), DispatchPendingJobs)
            .await
            .unwrap_err();

        assert!(matches!(err, DomainError::Forbidden(_)));
        assert!(registry.dispatched.lock().unwrap().is_empty());
        assert!(jobs.assigned().is_empty());
    }
}
