//! The pipeline's actions through the engine, on stub ports.

use super::*;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::caller::{CallerContext, ServiceIdentity};
use crate::domain::errors::DomainError;
use crate::domain::ids::{AppId, OrganizationId, PipelineId, ProjectId, TriggerId, UserId};
use crate::domain::job::JobOrigin;
use crate::domain::permission::Permission;
use crate::domain::pipeline::{Pipeline, PipelineName};
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService, actions};
use crate::test_support::pipelines::{PipelineBuilder, node};
use crate::test_support::projects::ProjectBuilder;
use crate::test_support::stubs::{
    EchoResolver, OneProject, StubJobs, StubRegistry, alice, empty_page,
};
use async_trait::async_trait;
use scylla_auth::authz::PermissionService;
use scylla_extension::Actions;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct StubPipelines {
    rows: Mutex<HashMap<PipelineId, Pipeline>>,
}

impl StubPipelines {
    fn page(&self, keep: impl Fn(&Pipeline) -> bool) -> DomainResult<PaginatedResult<Pipeline>> {
        let rows: Vec<Pipeline> = self
            .rows
            .lock()
            .unwrap()
            .values()
            .filter(|p| keep(p))
            .cloned()
            .collect();
        let total = rows.len() as u64;
        Ok(PaginatedResult::new(
            rows,
            &PaginationParams::default(),
            total,
        ))
    }
}

#[async_trait]
impl PipelineRepository for StubPipelines {
    async fn create(&self, pipeline: &Pipeline) -> DomainResult<Pipeline> {
        self.rows
            .lock()
            .unwrap()
            .insert(pipeline.id().clone(), pipeline.clone());
        Ok(pipeline.clone())
    }
    async fn find_by_id(&self, id: &PipelineId) -> DomainResult<Pipeline> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("Pipeline", id.to_string()))
    }
    async fn update(&self, pipeline: &Pipeline) -> DomainResult<Pipeline> {
        self.create(pipeline).await
    }
    async fn delete(&self, id: &PipelineId) -> DomainResult<()> {
        self.rows.lock().unwrap().remove(id);
        Ok(())
    }
    async fn list_all(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.page(|_| true)
    }
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.page(|p| p.project_id() == project_id)
    }
    async fn list_by_organization(
        &self,
        _: &OrganizationId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        empty_page()
    }
}

struct Lab {
    actions: Actions,
    uc: PipelineUseCases,
    pipelines: Arc<StubPipelines>,
    jobs: Arc<StubJobs>,
    registry: Arc<StubRegistry>,
}

impl Lab {
    async fn create(&self) -> DomainResult<Pipeline> {
        self.actions.run(&self.uc, &alice(), create()).await
    }

    fn seed(&self) -> Pipeline {
        let pipeline = PipelineBuilder::for_project_id(project_id()).build();
        self.pipelines
            .rows
            .lock()
            .unwrap()
            .insert(pipeline.id().clone(), pipeline.clone());
        pipeline
    }
}

fn lab(permissions: Arc<dyn PermissionService>) -> Lab {
    let pipelines = Arc::new(StubPipelines::default());
    let jobs = Arc::new(StubJobs::default());
    let registry = Arc::new(StubRegistry::default());
    let project = ProjectBuilder::for_org_id(OrganizationId::new("acme"), "rocket")
        .id(project_id())
        .build();
    Lab {
        actions: actions(permissions),
        uc: PipelineUseCases::new(
            pipelines.clone(),
            Arc::new(OneProject(project)),
            jobs.clone(),
            Arc::new(EchoResolver),
            Arc::new(DispatchUseCases::new(
                registry.clone(),
                Arc::new(RecordingPermissionService::new()),
                jobs.clone(),
                pipelines.clone(),
                Arc::new(EchoResolver),
            )),
        ),
        pipelines,
        jobs,
        registry,
    }
}

fn project_id() -> ProjectId {
    ProjectId::new("proj-1")
}

fn create() -> CreatePipeline {
    CreatePipeline {
        project_id: project_id(),
        name: PipelineName::new("build").unwrap(),
        nodes: vec![node("a", &[])],
    }
}

#[tokio::test]
async fn a_create_checks_the_permission_on_the_project_then_writes() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());

    let pipeline = lab.create().await.unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::CreatePipeline(project_id())]
    );
    assert!(
        lab.pipelines
            .rows
            .lock()
            .unwrap()
            .contains_key(pipeline.id())
    );
}

#[tokio::test]
async fn a_create_in_an_unknown_project_is_not_found_and_writes_nothing() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let err = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            CreatePipeline {
                project_id: ProjectId::new("ghost"),
                ..create()
            },
        )
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::NotFound { .. }));
    assert!(lab.pipelines.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_denied_caller_writes_nothing() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));

    let err = lab.create().await.unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.pipelines.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn an_update_checks_its_permission_and_changes_only_what_is_set() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let updated = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            UpdatePipeline {
                id: seeded.id().clone(),
                name: Some(PipelineName::new("deploy").unwrap()),
                nodes: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::UpdatePipeline(seeded.id().clone())]
    );
    assert_eq!(updated.name().as_str(), "deploy");
    assert_eq!(updated.nodes().len(), seeded.nodes().len());
}

#[tokio::test]
async fn a_delete_checks_its_permission_and_removes_the_row() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let deleted = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            DeletePipeline {
                id: seeded.id().clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::DeletePipeline(seeded.id().clone())]
    );
    assert_eq!(deleted.last_state().id(), seeded.id());
    assert!(lab.pipelines.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_get_and_a_project_list_check_their_permissions() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let got = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            GetPipeline {
                id: seeded.id().clone(),
            },
        )
        .await
        .unwrap();
    let page = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            ListProjectPipelines {
                project_id: project_id(),
                pagination: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(got.id(), seeded.id());
    assert_eq!(page.into_parts().0.len(), 1);
    assert_eq!(
        permissions.permissions(),
        vec![
            Permission::ReadPipeline(seeded.id().clone()),
            Permission::ListPipelinesByProject(project_id()),
        ]
    );
}

#[tokio::test]
async fn a_run_by_a_user_creates_a_job_with_a_human_origin_and_leaves_it_pending_without_agents() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let job = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            RunPipeline {
                id: seeded.id().clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::RunPipeline(seeded.id().clone())]
    );
    assert_eq!(
        job.origin(),
        &JobOrigin::Human {
            user_id: UserId::new("alice")
        }
    );
    assert!(job.inputs().is_empty());
    assert!(job.agent_app_id().is_none());
    assert_eq!(lab.jobs.rows().len(), 1);
    assert!(lab.registry.dispatched().is_empty());
    assert!(lab.jobs.assigned().is_empty());
}

#[tokio::test]
async fn a_run_hands_the_job_to_a_connected_agent_and_records_it() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    let seeded = lab.seed();
    let agent = AppId::new("agent-1");
    lab.registry.connect(&agent);

    let job = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            RunPipeline {
                id: seeded.id().clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(job.agent_app_id(), Some(&agent));
    assert_eq!(lab.registry.dispatched(), vec![agent.clone()]);
    assert_eq!(lab.jobs.assigned(), vec![(job.id().clone(), agent)]);
}

#[tokio::test]
async fn a_run_by_a_service_caller_is_forbidden_and_creates_no_job() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    let seeded = lab.seed();

    let err = lab
        .actions
        .run(
            &lab.uc,
            &CallerContext::Service(ServiceIdentity::recorder()),
            RunPipeline {
                id: seeded.id().clone(),
            },
        )
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.jobs.rows().is_empty());
}

#[tokio::test]
async fn a_denied_run_creates_no_job() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));
    let seeded = lab.seed();

    let err = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            RunPipeline {
                id: seeded.id().clone(),
            },
        )
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.jobs.rows().is_empty());
}

#[tokio::test]
async fn a_trigger_run_checks_run_pipeline_and_keeps_its_origin_and_inputs() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();
    let agent = AppId::new("agent-1");
    lab.registry.connect(&agent);
    let origin = JobOrigin::Cron {
        trigger_id: TriggerId::new("t-1"),
    };
    let inputs = vec![("MODE".to_string(), "nightly".to_string())];
    let runner = CallerContext::App(AppId::new("runner"));

    let job = lab
        .actions
        .run(
            &lab.uc,
            &runner,
            RunPipelineWithInputs {
                id: seeded.id().clone(),
                inputs: inputs.clone(),
                origin: origin.clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(
        permissions.checks(),
        vec![(runner, Permission::RunPipeline(seeded.id().clone()))]
    );
    assert_eq!(job.origin(), &origin);
    assert_eq!(job.inputs(), inputs.as_slice());
    assert_eq!(lab.jobs.assigned(), vec![(job.id().clone(), agent)]);
}
