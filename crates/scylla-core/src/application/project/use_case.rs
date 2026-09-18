use super::commands::{CreateProject, DeleteProject, SetProjectActive, UpdateProject};
use crate::application::pagination::{PaginatedResult, PaginationMetadata, PaginationParams};
use crate::application::{ProjectRepository, UserRepository};
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, ProjectId, UserId};
use crate::domain::permission::Permission;
use crate::domain::project::Project;
use crate::domain::user::User;
use derive_more::Constructor;
use scylla_auth::authz::{PermissionService, PolicyControl, Visibility, VisibilityResolver};
use scylla_extension::{Actions, Committed, Deleted};
use std::sync::Arc;
use tracing::instrument;

/// Writes go through `actions`: the permission check, the hooks and the two stages in
/// `prepare.rs` and `persist.rs`. Reads stay plain methods; they are not commands.
#[derive(Constructor)]
pub struct ProjectUseCases<
    P: ProjectRepository,
    U: UserRepository,
    PS: PermissionService,
    PC: PolicyControl,
> {
    pub(super) project_repo: Arc<P>,
    user_repo: Arc<U>,
    permission_service: Arc<PS>,
    visibility: Arc<dyn VisibilityResolver>,
    pub(super) policy_control: Arc<PC>,
    actions: Arc<Actions>,
}

impl<
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
> ProjectUseCases<P, U, PS, PC>
{
    #[instrument(skip_all, fields(name = %command.name, org_id = %command.organization_id))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        command: CreateProject,
    ) -> DomainResult<Project> {
        self.actions
            .send(self, caller, command)
            .await
            .map(Committed::into_outcome)
    }

    #[instrument(skip_all, fields(project_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &ProjectId) -> DomainResult<Project> {
        self.permission_service
            .check(caller, Permission::ReadProject(id.clone()))
            .await?;
        self.project_repo.find_by_id(id).await
    }

    #[instrument(skip_all, fields(project_id = %command.id))]
    pub async fn update(
        &self,
        caller: &CallerContext,
        command: UpdateProject,
    ) -> DomainResult<Project> {
        self.actions
            .send(self, caller, command)
            .await
            .map(Committed::into_outcome)
    }

    #[instrument(skip_all, fields(project_id = %command.id, is_active = command.is_active))]
    pub async fn set_active(
        &self,
        caller: &CallerContext,
        command: SetProjectActive,
    ) -> DomainResult<Project> {
        self.actions
            .send(self, caller, command)
            .await
            .map(Committed::into_outcome)
    }

    #[instrument(skip_all, fields(project_id = %command.id))]
    pub async fn delete(
        &self,
        caller: &CallerContext,
        command: DeleteProject,
    ) -> DomainResult<Deleted<Project>> {
        self.actions
            .send(self, caller, command)
            .await
            .map(Committed::into_outcome)
    }

    #[instrument(skip(self, caller, pagination))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        self.permission_service
            .check(caller, Permission::ListProjects)
            .await?;
        self.project_repo.list_all(pagination).await
    }

    /// Not gated on `listProjectsByOrganization`: a project-only role must see its own project, not be refused.
    #[instrument(skip_all, fields(organization_id = %organization_id))]
    pub async fn list_by_organization(
        &self,
        caller: &CallerContext,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        self.permission_service
            .check(
                caller,
                Permission::ReadOrganization(organization_id.clone()),
            )
            .await?;

        let visible = if self
            .permission_service
            .check(
                caller,
                Permission::ListProjectsByOrganization(organization_id.clone()),
            )
            .await
            .is_ok()
        {
            Visibility::All
        } else {
            self.visibility
                .visible_scopes(caller, Permission::ReadProject(ProjectId::new("_")).key())
                .await?
        };

        self.project_repo
            .list_by_organization(organization_id, pagination, &visible)
            .await
    }

    #[instrument(skip_all, fields(project_id = %project_id))]
    pub async fn list_users(
        &self,
        caller: &CallerContext,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<User>, PaginationMetadata)> {
        self.permission_service
            .check(caller, Permission::ListProjectMembers(project_id.clone()))
            .await?;

        let paginated = self
            .project_repo
            .list_principals(project_id, pagination)
            .await?;
        let (user_ids, metadata) = paginated.into_parts();

        let mut by_id: std::collections::HashMap<String, User> = self
            .user_repo
            .find_by_ids(&user_ids)
            .await?
            .into_iter()
            .map(|u| (u.id().as_str().to_owned(), u))
            .collect();
        let users = user_ids
            .iter()
            .filter_map(|id| by_id.remove(id.as_str()))
            .collect();

        Ok((users, metadata))
    }

    #[instrument(skip_all, fields(user_id = %user_id))]
    pub async fn list_user_projects(
        &self,
        caller: &CallerContext,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<Project>, PaginationMetadata)> {
        self.permission_service
            .check(caller, Permission::ListUserProjects(user_id.clone()))
            .await?;

        let paginated = self.project_repo.list_for_user(user_id, pagination).await?;
        Ok(paginated.into_parts())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::PermissionAuthorizer;
    use crate::domain::errors::DomainError;
    use crate::domain::project::ProjectName;
    use crate::domain::user::{Email, User, Username};
    use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService};
    use async_trait::async_trait;
    use scylla_auth::authz::Grant;
    use scylla_extension::{Action, Hooks, Policy, StageKind};
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct StubProjects {
        rows: Mutex<HashMap<ProjectId, Project>>,
        grants: Mutex<Vec<Grant>>,
    }

    fn empty<T>() -> DomainResult<PaginatedResult<T>> {
        Ok(PaginatedResult::new(
            Vec::new(),
            &PaginationParams::default(),
            0,
        ))
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
        async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
            self.rows.lock().unwrap().remove(id);
            Ok(())
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

    struct StubUsers;

    #[async_trait]
    impl UserRepository for StubUsers {
        async fn create(&self, _: &User) -> DomainResult<User> {
            unreachable!("no user write in a project action")
        }
        async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
            Err(DomainError::not_found("User", id.to_string()))
        }
        async fn find_by_ids(&self, _: &[UserId]) -> DomainResult<Vec<User>> {
            Ok(Vec::new())
        }
        async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
            Err(DomainError::not_found("User", username.to_string()))
        }
        async fn find_by_email(&self, email: &Email) -> DomainResult<User> {
            Err(DomainError::not_found("User", email.to_string()))
        }
        async fn update(&self, _: &User) -> DomainResult<User> {
            unreachable!("no user write in a project action")
        }
        async fn delete(&self, _: &UserId) -> DomainResult<()> {
            unreachable!("no user write in a project action")
        }
        async fn list_all(
            &self,
            _: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<User>> {
            empty()
        }
        async fn username_exists(&self, _: &Username) -> DomainResult<bool> {
            Ok(false)
        }
    }

    #[derive(Default)]
    struct StubPolicy {
        reloads: Mutex<usize>,
    }

    #[async_trait]
    impl PolicyControl for StubPolicy {
        async fn reload(&self) -> DomainResult<()> {
            *self.reloads.lock().unwrap() += 1;
            Ok(())
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
                action.permission().key()
            )))
        }
    }

    struct Lab<PS: PermissionService> {
        uc: ProjectUseCases<StubProjects, StubUsers, PS, StubPolicy>,
        projects: Arc<StubProjects>,
        policy: Arc<StubPolicy>,
    }

    fn lab<PS: PermissionService + 'static>(permissions: Arc<PS>, hooks: Hooks) -> Lab<PS> {
        let projects = Arc::new(StubProjects::default());
        let policy = Arc::new(StubPolicy::default());
        let actions = Arc::new(Actions::new(
            Arc::new(PermissionAuthorizer::new(permissions.clone())),
            Arc::new(hooks),
        ));
        Lab {
            uc: ProjectUseCases::new(
                projects.clone(),
                Arc::new(StubUsers),
                permissions,
                Arc::new(StubVisibility),
                policy.clone(),
                actions,
            ),
            projects,
            policy,
        }
    }

    fn alice() -> CallerContext {
        CallerContext::User(UserId::new("alice"))
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

        let project = lab.uc.create(&alice(), create("rocket")).await.unwrap();

        assert_eq!(
            permissions.permissions(),
            vec![Permission::CreateProject(OrganizationId::new("acme"))]
        );
        assert!(lab.projects.rows.lock().unwrap().contains_key(project.id()));
        assert_eq!(lab.projects.grants.lock().unwrap().len(), 1);
        assert_eq!(*lab.policy.reloads.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn a_denied_caller_writes_nothing() {
        let lab = lab(Arc::new(DenyingPermissionService::new()), Hooks::new());

        let err = lab.uc.create(&alice(), create("rocket")).await.unwrap_err();

        assert!(matches!(err, DomainError::Forbidden(_)));
        assert!(lab.projects.rows.lock().unwrap().is_empty());
        assert_eq!(*lab.policy.reloads.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn a_policy_in_the_hooks_vetoes_after_the_permission_check() {
        let permissions = Arc::new(RecordingPermissionService::new());
        let mut hooks = Hooks::new();
        hooks.policy(StageKind::Prepare, Arc::new(Veto));
        let lab = lab(permissions.clone(), hooks);

        let err = lab.uc.create(&alice(), create("rocket")).await.unwrap_err();

        assert!(matches!(&err, DomainError::QuotaExceeded(m) if m == "vetoed createProject"));
        assert_eq!(permissions.permissions().len(), 1);
        assert!(lab.projects.rows.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn an_update_stages_the_change_and_persists_it() {
        let permissions = Arc::new(RecordingPermissionService::new());
        let lab = lab(permissions.clone(), Hooks::new());
        let created = lab.uc.create(&alice(), create("old")).await.unwrap();

        let updated = lab
            .uc
            .update(
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
        let created = lab.uc.create(&alice(), create("gone")).await.unwrap();

        let deleted = lab
            .uc
            .delete(
                &alice(),
                DeleteProject {
                    id: created.id().clone(),
                },
            )
            .await
            .unwrap();

        assert_eq!(deleted.last_state().id(), created.id());
        assert!(lab.projects.rows.lock().unwrap().is_empty());
        assert_eq!(*lab.policy.reloads.lock().unwrap(), 2);
    }
}
