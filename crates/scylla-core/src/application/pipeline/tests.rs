//! The pipeline's actions through the engine, on stub ports.

use super::*;
use crate::application::PermissionAuthorizer;
use crate::application::agent::DispatchNode;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::caller::ServiceIdentity;
use crate::domain::errors::DomainError;
use crate::domain::ids::{OrganizationId, ProjectId, TriggerId, UserId};
use crate::domain::pipeline::{Pipeline, PipelineName, PipelineNode};
use crate::domain::project::Project;
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService};
use crate::test_support::pipelines::{PipelineBuilder, node};
use crate::test_support::projects::ProjectBuilder;
use async_trait::async_trait;
use scylla_auth::authz::{Grant, Visibility};
use scylla_extension::{Actions, Hooks};
use std::collections::HashMap;
use std::sync::Mutex;

fn empty<T>() -> DomainResult<PaginatedResult<T>> {
    Ok(PaginatedResult::new(
        Vec::new(),
        &PaginationParams::default(),
        0,
    ))
}

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
        empty()
    }
}

struct StubProjects {
    project: Project,
}

#[async_trait]
impl ProjectRepository for StubProjects {
    async fn create(&self, _: &Project) -> DomainResult<Project> {
        unreachable!("no project write in a pipeline action")
    }
    async fn provision_with_owner(&self, _: &Project, _: &Grant) -> DomainResult<()> {
        unreachable!("no project write in a pipeline action")
    }
    async fn list_principals(
        &self,
        _: &ProjectId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        empty()
    }
    async fn list_for_user(
        &self,
        _: &UserId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty()
    }
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        if id == self.project.id() {
            Ok(self.project.clone())
        } else {
            Err(DomainError::not_found("Project", id.to_string()))
        }
    }
    async fn find_by_ids(&self, _: &[ProjectId]) -> DomainResult<Vec<Project>> {
        Ok(Vec::new())
    }
    async fn update(&self, _: &Project) -> DomainResult<Project> {
        unreachable!("no project write in a pipeline action")
    }
    async fn delete(&self, _: &Project) -> DomainResult<()> {
        unreachable!("no project write in a pipeline action")
    }
    async fn list_all(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty()
    }
    async fn list_active(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty()
    }
    async fn list_by_organization(
        &self,
        _: &OrganizationId,
        _: Option<&PaginationParams>,
        _: &Visibility,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty()
    }
}

#[derive(Default)]
struct StubJobs {
    rows: Mutex<Vec<Job>>,
    assigned: Mutex<Vec<(JobId, AppId)>>,
}

#[async_trait]
impl JobRepository for StubJobs {
    async fn create(&self, job: &Job) -> DomainResult<Job> {
        self.rows.lock().unwrap().push(job.clone());
        Ok(job.clone())
    }
    async fn find_by_id(&self, id: &JobId) -> DomainResult<Job> {
        Err(DomainError::not_found("Job", id.to_string()))
    }
    async fn update(&self, _: &Job) -> DomainResult<Job> {
        unreachable!("no job update in a pipeline action")
    }
    async fn set_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()> {
        self.assigned
            .lock()
            .unwrap()
            .push((job_id.clone(), app_id.clone()));
        Ok(())
    }
    async fn list_pending_unassigned(&self) -> DomainResult<Vec<Job>> {
        Ok(Vec::new())
    }
    async fn orphan_running_without_agents(&self, _: &[AppId]) -> DomainResult<u64> {
        Ok(0)
    }
    async fn delete(&self, _: &JobId) -> DomainResult<()> {
        unreachable!("no job delete in a pipeline action")
    }
    async fn list_all(&self, _: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Job>> {
        empty()
    }
    async fn list_by_pipeline(
        &self,
        _: &PipelineId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        empty()
    }
    async fn list_by_project(
        &self,
        _: &ProjectId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        empty()
    }
    async fn list_by_organization(
        &self,
        _: &OrganizationId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        empty()
    }
}

struct StubResolver;

#[async_trait]
impl SecretResolver for StubResolver {
    async fn resolve(
        &self,
        _: &ProjectId,
        nodes: &[PipelineNode],
    ) -> DomainResult<Vec<DispatchNode>> {
        Ok(nodes
            .iter()
            .map(|n| DispatchNode {
                id: n.id().to_string(),
                deps: Vec::new(),
                working_dir: None,
                step: n.step().clone(),
                env: Vec::new(),
            })
            .collect())
    }
}

struct Lab {
    actions: Actions,
    uc: PipelineUseCases,
    pipelines: Arc<StubPipelines>,
    jobs: Arc<StubJobs>,
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
    let project = ProjectBuilder::for_org_id(OrganizationId::new("acme"), "rocket")
        .id(project_id())
        .build();
    Lab {
        actions: Actions::new(
            Arc::new(PermissionAuthorizer::new(permissions.clone())),
            Arc::new(Hooks::new()),
        ),
        uc: PipelineUseCases::new(
            pipelines.clone(),
            Arc::new(StubProjects { project }),
            jobs.clone(),
            permissions,
            Arc::new(StubResolver),
        ),
        pipelines,
        jobs,
    }
}

fn alice() -> CallerContext {
    CallerContext::User(UserId::new("alice"))
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
async fn a_run_by_a_user_creates_a_job_with_a_human_origin_and_its_dispatch() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let (job, dispatch) = lab
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
    assert_eq!(dispatch.job_id, job.id().to_string());
    assert_eq!(dispatch.nodes.len(), 1);
    assert_eq!(lab.jobs.rows.lock().unwrap().len(), 1);
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
    assert!(lab.jobs.rows.lock().unwrap().is_empty());
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
    assert!(lab.jobs.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_trigger_run_checks_run_pipeline_and_keeps_its_origin_and_inputs() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();
    let origin = JobOrigin::Cron {
        trigger_id: TriggerId::new("t-1"),
    };
    let inputs = [("MODE".to_string(), "nightly".to_string())];

    let (job, _) = lab
        .uc
        .run_with_inputs(
            &CallerContext::App(AppId::new("runner")),
            seeded.id(),
            &inputs,
            origin.clone(),
        )
        .await
        .unwrap();
    lab.uc
        .assign_agent(job.id(), &AppId::new("agent-1"))
        .await
        .unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::RunPipeline(seeded.id().clone())]
    );
    assert_eq!(job.origin(), &origin);
    assert_eq!(job.inputs(), &inputs);
    assert_eq!(
        lab.jobs.assigned.lock().unwrap().as_slice(),
        &[(job.id().clone(), AppId::new("agent-1"))]
    );
}
