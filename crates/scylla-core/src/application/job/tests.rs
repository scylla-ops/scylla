//! The job's actions through the engine, on a stub port.

use super::*;
use crate::application::PermissionAuthorizer;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, JobId, OrganizationId, PipelineId, ProjectId};
use crate::domain::job::{Job, JobStatus};
use crate::domain::permission::Permission;
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService};
use crate::test_support::jobs::job;
use crate::test_support::pipelines::PipelineBuilder;
use async_trait::async_trait;
use scylla_auth::authz::PermissionService;
use scylla_extension::{Actions, Hooks};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct StubJobs {
    rows: Mutex<HashMap<JobId, Job>>,
    deletes: Mutex<usize>,
}

fn empty<T>() -> DomainResult<PaginatedResult<T>> {
    Ok(PaginatedResult::new(
        Vec::new(),
        &PaginationParams::default(),
        0,
    ))
}

#[async_trait]
impl JobRepository for StubJobs {
    async fn create(&self, job: &Job) -> DomainResult<Job> {
        self.rows
            .lock()
            .unwrap()
            .insert(job.id().clone(), job.clone());
        Ok(job.clone())
    }
    async fn find_by_id(&self, id: &JobId) -> DomainResult<Job> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("Job", id.to_string()))
    }
    async fn update(&self, job: &Job) -> DomainResult<Job> {
        self.create(job).await
    }
    async fn set_agent(&self, _: &JobId, _: &AppId) -> DomainResult<()> {
        unreachable!("no agent assignment in a job action")
    }
    async fn list_pending_unassigned(&self) -> DomainResult<Vec<Job>> {
        unreachable!("the scheduler reads pending jobs, not a job action")
    }
    async fn orphan_running_without_agents(&self, _: &[AppId]) -> DomainResult<u64> {
        unreachable!("the reaper orphans jobs, not a job action")
    }
    async fn delete(&self, id: &JobId) -> DomainResult<()> {
        *self.deletes.lock().unwrap() += 1;
        self.rows.lock().unwrap().remove(id);
        Ok(())
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

struct Lab {
    actions: Actions,
    uc: JobUseCases<StubJobs>,
    jobs: Arc<StubJobs>,
}

impl Lab {
    fn seed(&self) -> Job {
        let pipeline = PipelineBuilder::for_project_id(ProjectId::new("p")).build();
        let job = job(&pipeline);
        self.jobs
            .rows
            .lock()
            .unwrap()
            .insert(job.id().clone(), job.clone());
        job
    }
}

fn lab<PS: PermissionService + 'static>(permissions: Arc<PS>) -> Lab {
    let jobs = Arc::new(StubJobs::default());
    Lab {
        actions: Actions::new(
            Arc::new(PermissionAuthorizer::new(permissions)),
            Arc::new(Hooks::new()),
        ),
        uc: JobUseCases::new(jobs.clone()),
        jobs,
    }
}

fn agent() -> CallerContext {
    CallerContext::App(AppId::new("agent"))
}

#[tokio::test]
async fn a_status_event_checks_write_job_status_and_persists_the_transition() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let job = lab
        .actions
        .run(
            &lab.uc,
            &agent(),
            RecordJobStatus {
                job_id: seeded.id().clone(),
                event: JobEvent::JobStarted,
            },
        )
        .await
        .unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::WriteJobStatus(seeded.id().clone())]
    );
    assert_eq!(job.status(), JobStatus::Running);
    assert_eq!(
        lab.jobs.rows.lock().unwrap()[seeded.id()].status(),
        JobStatus::Running
    );
}

#[tokio::test]
async fn a_denied_status_event_writes_nothing() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));
    let seeded = lab.seed();

    let err = lab
        .actions
        .run(
            &lab.uc,
            &agent(),
            RecordJobStatus {
                job_id: seeded.id().clone(),
                event: JobEvent::JobStarted,
            },
        )
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert_eq!(
        lab.jobs.rows.lock().unwrap()[seeded.id()].status(),
        JobStatus::Pending
    );
}

#[tokio::test]
async fn a_delete_returns_the_tombstone() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let deleted = lab
        .actions
        .run(
            &lab.uc,
            &agent(),
            DeleteJob {
                id: seeded.id().clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(deleted.last_state().id(), seeded.id());
    assert!(lab.jobs.rows.lock().unwrap().is_empty());
    assert_eq!(
        permissions.permissions(),
        vec![Permission::DeleteJob(seeded.id().clone())]
    );
}

#[tokio::test]
async fn a_delete_of_a_missing_job_is_not_found_and_deletes_nothing() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let err = lab
        .actions
        .run(
            &lab.uc,
            &agent(),
            DeleteJob {
                id: JobId::new("gone"),
            },
        )
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::NotFound { .. }));
    assert_eq!(*lab.jobs.deletes.lock().unwrap(), 0);
}

#[tokio::test]
async fn each_read_checks_its_own_permission() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();
    let caller = agent();

    let got = lab
        .actions
        .run(
            &lab.uc,
            &caller,
            GetJob {
                id: seeded.id().clone(),
            },
        )
        .await
        .unwrap();
    lab.actions
        .run(&lab.uc, &caller, ListJobs { pagination: None })
        .await
        .unwrap();
    lab.actions
        .run(
            &lab.uc,
            &caller,
            ListPipelineJobs {
                pipeline_id: PipelineId::new("pipe"),
                pagination: None,
            },
        )
        .await
        .unwrap();
    lab.actions
        .run(
            &lab.uc,
            &caller,
            ListProjectJobs {
                project_id: ProjectId::new("p"),
                pagination: None,
            },
        )
        .await
        .unwrap();
    lab.actions
        .run(
            &lab.uc,
            &caller,
            ListOrganizationJobs {
                organization_id: OrganizationId::new("acme"),
                pagination: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(got.id(), seeded.id());
    assert_eq!(
        permissions.permissions(),
        vec![
            Permission::ReadJob(seeded.id().clone()),
            Permission::ListJobs,
            Permission::ListJobsByPipeline(PipelineId::new("pipe")),
            Permission::ListJobsByProject(ProjectId::new("p")),
            Permission::ListJobsByOrganization(OrganizationId::new("acme")),
        ]
    );
}
