use crate::application::pagination::{PaginatedResult, PaginationMetadata, PaginationParams};
use crate::application::{ProjectRepository, UserRepository, quota};
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, ProjectId, UserId};
use crate::domain::permission::Permission;
use crate::domain::project::Project;
use crate::domain::project::{ProjectDescription, ProjectName};
use crate::domain::role::RoleName;
use crate::domain::user::User;
use derive_more::Constructor;
use scylla_auth::authz::{
    Grant, PROJECT_ADMIN_ROLE, PermissionService, PolicyControl, Principal, Scope, Visibility,
    VisibilityResolver,
};
use scylla_extension::{QuotaPolicy, Resource};
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct ProjectUseCases<
    P: ProjectRepository,
    U: UserRepository,
    PS: PermissionService,
    PC: PolicyControl,
> {
    project_repo: Arc<P>,
    user_repo: Arc<U>,
    permission_service: Arc<PS>,
    visibility: Arc<dyn VisibilityResolver>,
    policy_control: Arc<PC>,
    quota: Arc<dyn QuotaPolicy>,
}

impl<P: ProjectRepository, U: UserRepository, PS: PermissionService, PC: PolicyControl>
    ProjectUseCases<P, U, PS, PC>
{
    #[instrument(skip_all, fields(name = %name, org_id = %organization_id))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        name: ProjectName,
        description: Option<ProjectDescription>,
        organization_id: OrganizationId,
    ) -> DomainResult<Project> {
        self.permission_service
            .check(caller, Permission::CreateProject(organization_id.clone()))
            .await?;

        quota::enforce(
            self.quota
                .check(Resource::Project, organization_id.as_str())
                .await,
        )?;

        let project = Project::create(name, description, organization_id)?;

        match caller {
            CallerContext::User(user_id) => {
                let role = RoleName::new(PROJECT_ADMIN_ROLE)?;
                let grant = Grant::new(
                    Principal::User(user_id.clone()),
                    role,
                    Scope::Project(project.id().clone()),
                );
                self.project_repo
                    .provision_with_owner(&project, &grant)
                    .await?;
                self.policy_control.reload().await?;
                Ok(project)
            }
            _ => self.project_repo.create(&project).await,
        }
    }

    #[instrument(skip_all, fields(project_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &ProjectId) -> DomainResult<Project> {
        self.permission_service
            .check(caller, Permission::ReadProject(id.clone()))
            .await?;
        self.project_repo.find_by_id(id).await
    }

    #[instrument(skip_all, fields(project_id = %id))]
    pub async fn update(
        &self,
        caller: &CallerContext,
        id: &ProjectId,
        name: Option<ProjectName>,
        description: Option<Option<ProjectDescription>>,
    ) -> DomainResult<Project> {
        self.permission_service
            .check(caller, Permission::UpdateProject(id.clone()))
            .await?;

        let mut project = self.project_repo.find_by_id(id).await?;

        if let Some(new_name) = name {
            project.update_name(new_name)?;
        }
        if let Some(new_desc) = description {
            project.update_description(new_desc)?;
        }

        self.project_repo.update(&project).await
    }

    #[instrument(skip_all, fields(project_id = %id))]
    pub async fn set_active(
        &self,
        caller: &CallerContext,
        id: &ProjectId,
        is_active: bool,
    ) -> DomainResult<Project> {
        self.permission_service
            .check(caller, Permission::UpdateProject(id.clone()))
            .await?;

        let mut project = self.project_repo.find_by_id(id).await?;
        project.set_active(is_active);
        self.project_repo.update(&project).await?;
        Ok(project)
    }

    #[instrument(skip_all, fields(project_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &ProjectId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeleteProject(id.clone()))
            .await?;
        self.project_repo.find_by_id(id).await?;
        // A DB trigger drops the project's grants with the row; reload so the live set stops carrying them.
        self.project_repo.delete(id).await?;
        self.policy_control.reload().await
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
