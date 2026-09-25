use super::*;
use crate::domain::caller::ServiceIdentity;
use crate::domain::errors::DomainResult;
use crate::domain::pipeline::Pipeline;
use crate::test_support::authz::{RecordingPermissionService, actions};
use crate::test_support::jobs::job;
use crate::test_support::organizations::org;
use crate::test_support::pipelines::pipeline;
use crate::test_support::projects::project;
use crate::test_support::stubs::{EchoResolver, OnePipeline, StubJobs, StubRegistry, alice};
use async_trait::async_trait;

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

fn registry(connected: &[&str]) -> Arc<StubRegistry> {
    let registry = StubRegistry::accepting();
    for id in connected {
        registry.connect(&AppId::new(*id));
    }
    Arc::new(registry)
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
    let registry = registry(&["app-unauthorized", "app-ok"]);
    let uc = uc(
        registry.clone(),
        Arc::new(StubPerms { allowed: "app-ok" }),
        Arc::default(),
        a_pipeline(),
    );

    let outcome = uc.dispatch_job(&PipelineId::new("pl1"), &dispatch()).await;

    assert!(matches!(outcome, DispatchOutcome::Dispatched(id) if id.as_str() == "app-ok"));
    assert_eq!(registry.dispatched_to(), [AppId::new("app-ok")]);
}

#[tokio::test]
async fn spreads_jobs_round_robin_across_authorized_agents() {
    let registry = registry(&["app-a", "app-b"]);
    let uc = uc(
        registry.clone(),
        Arc::new(RecordingPermissionService::new()),
        Arc::default(),
        a_pipeline(),
    );

    for _ in 0..4 {
        uc.dispatch_job(&PipelineId::new("pl1"), &dispatch()).await;
    }

    let dispatched = registry.dispatched_to();
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
    let registry = registry(&["app-busy", "app-idle"]);
    registry.load(&AppId::new("app-busy"), 3);
    let uc = uc(
        registry.clone(),
        Arc::new(RecordingPermissionService::new()),
        Arc::default(),
        a_pipeline(),
    );

    uc.dispatch_job(&PipelineId::new("pl1"), &dispatch()).await;

    assert_eq!(registry.dispatched_to(), [AppId::new("app-idle")]);
}

#[tokio::test]
async fn no_agent_when_none_authorized() {
    let registry = registry(&["app-x"]);
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
    let registry = registry(&["agent-1"]);
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
    let registry = registry(&["agent-1"]);
    let permissions = Arc::new(RecordingPermissionService::new());
    let uc = uc(registry.clone(), permissions.clone(), jobs.clone(), pl);

    let err = actions(permissions)
        .run(&uc, &alice(), DispatchPendingJobs)
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(registry.dispatched().is_empty());
    assert!(jobs.assigned().is_empty());
}
