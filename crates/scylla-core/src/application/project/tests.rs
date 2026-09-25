//! The project's actions through the engine, on stub ports.

use super::*;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::application::project::{CreateProject, DeleteProject, GetProject, UpdateProject};
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{OrganizationId, ProjectId, UserId};
use crate::domain::permission::Permission;
use crate::domain::project::{Project, ProjectName};
use crate::test_support::authz::{
    DenyingPermissionService, RecordingPermissionService, actions_with,
};
use crate::test_support::stubs::{CountingPolicy, NoUsers, alice, empty_page};
use async_trait::async_trait;
use scylla_auth::authz::{Grant, Visibility};
use scylla_extension::{Action, Actions, Hooks, Policy, StageKind};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct StubProjects {
    rows: Mutex<HashMap<ProjectId, Project>>,
    grants: Mutex<Vec<Grant>>,
}

#[async_trait]
impl ProjectRepository for StubProjects {
    async fn create(&self, project: &Project) -> DomainResult<Project> {
        self.rows
            .lock()
            .unwrap()
            .insert(project.id().clone(), project.clone());
        Ok(project.clone())
    }
    async fn provision_with_owner(&self, project: &Project, grant: &Grant) -> DomainResult<()> {
        self.create(project).await?;
        self.grants.lock().unwrap().push(grant.clone());
        Ok(())
    }
    async fn list_principals(
        &self,
        _: &ProjectId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        empty_page()
    }
    async fn list_for_user(
        &self,
        _: &UserId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty_page()
    }
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("Project", id.to_string()))
    }
    async fn find_by_ids(&self, _: &[ProjectId]) -> DomainResult<Vec<Project>> {
        Ok(Vec::new())
    }
    async fn update(&self, project: &Project) -> DomainResult<Project> {
        self.create(project).await
    }
    async fn delete(&self, project: &Project) -> DomainResult<()> {
        self.rows.lock().unwrap().remove(project.id());
        Ok(())
    }
    async fn list_all(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty_page()
    }
    async fn list_active(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty_page()
    }
    async fn list_by_organization(
        &self,
        _: &OrganizationId,
        _: Option<&PaginationParams>,
        _: &Visibility,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty_page()
    }
}

struct StubVisibility;

#[async_trait]
impl VisibilityResolver for StubVisibility {
    async fn visible_scopes(&self, _: &CallerContext, _: &str) -> DomainResult<Visibility> {
        Ok(Visibility::All)
    }
}

struct Veto;

#[async_trait]
impl Policy for Veto {
    async fn enforce(&self, _: StageKind, action: &dyn Action) -> DomainResult<()> {
        Err(DomainError::quota_exceeded(format!(
            "vetoed {}",
            action.access()
        )))
    }
}

struct Lab {
    actions: Actions,
    uc: ProjectUseCases,
    projects: Arc<StubProjects>,
    policy: Arc<CountingPolicy>,
}

impl Lab {
    async fn create(&self, name: &str) -> DomainResult<Project> {
        self.actions.run(&self.uc, &alice(), create(name)).await
    }
}

fn lab(permissions: Arc<dyn PermissionService>, hooks: Hooks) -> Lab {
    let projects = Arc::new(StubProjects::default());
    let policy = Arc::new(CountingPolicy::default());
    Lab {
        actions: actions_with(permissions.clone(), hooks),
        uc: ProjectUseCases::new(
            projects.clone(),
            Arc::new(NoUsers),
            permissions,
            Arc::new(StubVisibility),
            policy.clone(),
        ),
        projects,
        policy,
    }
}

fn create(name: &str) -> CreateProject {
    CreateProject {
        organization_id: OrganizationId::new("acme"),
        name: ProjectName::new(name).unwrap(),
        description: None,
    }
}

#[tokio::test]
async fn a_create_by_a_user_checks_the_permission_then_writes_the_owner_grant() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone(), Hooks::new());

    let project = lab.create("rocket").await.unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::CreateProject(OrganizationId::new("acme"))]
    );
    assert!(lab.projects.rows.lock().unwrap().contains_key(project.id()));
    assert_eq!(lab.projects.grants.lock().unwrap().len(), 1);
    assert_eq!(lab.policy.reloads(), 1);
}

#[tokio::test]
async fn a_denied_caller_writes_nothing() {
    let lab = lab(Arc::new(DenyingPermissionService::new()), Hooks::new());

    let err = lab.create("rocket").await.unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.projects.rows.lock().unwrap().is_empty());
    assert_eq!(lab.policy.reloads(), 0);
}

#[tokio::test]
async fn a_policy_in_the_hooks_vetoes_after_the_permission_check() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let mut hooks = Hooks::new();
    hooks.policy(StageKind::Prepare, Arc::new(Veto));
    let lab = lab(permissions.clone(), hooks);

    let err = lab.create("rocket").await.unwrap_err();

    assert!(matches!(&err, DomainError::QuotaExceeded(m) if m == "vetoed createProject"));
    assert_eq!(permissions.permissions().len(), 1);
    assert!(lab.projects.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn an_update_stages_the_change_and_persists_it() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone(), Hooks::new());
    let created = lab.create("old").await.unwrap();

    let updated = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            UpdateProject {
                id: created.id().clone(),
                name: Some(ProjectName::new("new").unwrap()),
                description: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.name().as_str(), "new");
    assert_eq!(
        lab.projects.rows.lock().unwrap()[created.id()]
            .name()
            .as_str(),
        "new"
    );
    assert_eq!(
        permissions.permissions()[1],
        Permission::UpdateProject(created.id().clone())
    );
}

#[tokio::test]
async fn a_delete_returns_the_tombstone_and_reloads_the_policies() {
    let lab = lab(Arc::new(RecordingPermissionService::new()), Hooks::new());
    let created = lab.create("gone").await.unwrap();

    let deleted = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            DeleteProject {
                id: created.id().clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(deleted.last_state().id(), created.id());
    assert!(lab.projects.rows.lock().unwrap().is_empty());
    assert_eq!(lab.policy.reloads(), 2);
}

#[tokio::test]
async fn a_read_checks_its_permission_and_takes_the_same_hooks() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let mut hooks = Hooks::new();
    hooks.policy(StageKind::Fetch, Arc::new(Veto));
    let lab = lab(permissions.clone(), hooks);
    let created = lab.create("seen").await.unwrap();

    let err = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            GetProject {
                id: created.id().clone(),
            },
        )
        .await
        .unwrap_err();

    assert!(matches!(&err, DomainError::QuotaExceeded(m) if m == "vetoed readProject"));
    assert_eq!(
        permissions.permissions()[1],
        Permission::ReadProject(created.id().clone())
    );
}
