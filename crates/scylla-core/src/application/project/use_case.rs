use crate::application::authz::grant::{
    Grant, GrantRepository, PROJECT_ADMIN_ROLE, Principal, Scope, removal_orphans_scope,
};
use crate::application::authz::policy::PolicyControl;
use crate::application::caller::CallerContext;
use crate::application::{
    PermissionService, ProjectRepository, UserProjectRepository, UserRepository,
};
use crate::domain::entities::{OrganizationId, Project, ProjectId, User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::project::{ProjectDescription, ProjectName};
use crate::domain::value_objects::role::RoleName;
use crate::domain::value_objects::{PaginatedResult, PaginationMetadata, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct ProjectUseCases<
    P: ProjectRepository,
    UP: UserProjectRepository,
    U: UserRepository,
    PS: PermissionService,
    PC: PolicyControl,
> {
    project_repo: Arc<P>,
    user_project_repo: Arc<UP>,
    user_repo: Arc<U>,
    grant_repo: Arc<dyn GrantRepository>,
    permission_service: Arc<PS>,
    policy_control: Arc<PC>,
    /// Per-org limits enforced on project creation.
    quotas: crate::application::quota::Quotas,
}

impl<
    P: ProjectRepository,
    UP: UserProjectRepository,
    U: UserRepository,
    PS: PermissionService,
    PC: PolicyControl,
> ProjectUseCases<P, UP, U, PS, PC>
{
    #[instrument(skip(self, caller), fields(name = %name, org_id = %organization_id))]
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

        // Cap projects per organization.
        let used = self
            .project_repo
            .count_by_organization(&organization_id)
            .await?;
        if used >= self.quotas.max_projects_per_org {
            return Err(DomainError::quota_exceeded(format!(
                "project quota reached for this organization ({used}/{})",
                self.quotas.max_projects_per_org
            )));
        }

        let project = Project::create(name, description, organization_id)?;

        // Enroll the human creator as a member + project-admin of the project they
        // just created — mirrors organization create. The project insert,
        // membership and owner grant happen in ONE transaction so a partial
        // failure can never leave a project without an owner. Machine/anonymous
        // callers have no human to enroll, so they just get the bare project.
        match caller {
            CallerContext::User(user_id) => {
                let role = RoleName::new(PROJECT_ADMIN_ROLE)?;
                let grant = Grant::new(
                    Principal::User(user_id.clone()),
                    role,
                    Scope::Project(project.id().clone()),
                );
                self.project_repo
                    .provision_with_owner(&project, user_id, &grant)
                    .await?;
                // Make the project-admin grant live now so the creator can act on
                // the project immediately, without a control-plane restart.
                self.policy_control.reload().await?;
                Ok(project)
            }
            _ => self.project_repo.create(&project).await,
        }
    }

    #[instrument(skip(self, caller), fields(project_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &ProjectId) -> DomainResult<Project> {
        self.permission_service
            .check(caller, Permission::ReadProject(id.clone()))
            .await?;
        self.project_repo.find_by_id(id).await
    }

    #[instrument(skip(self, caller), fields(project_id = %id))]
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

    /// Set the active flag to an explicit value and return the updated project.
    /// Idempotent, so a retried call is safe — see [`Project::set_active`].
    #[instrument(skip(self, caller), fields(project_id = %id))]
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

    #[instrument(skip(self, caller), fields(project_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &ProjectId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeleteProject(id.clone()))
            .await?;
        self.project_repo.find_by_id(id).await?;
        // A DB trigger drops the project-scoped grants with the row; reload so
        // the live policy set stops carrying their dead links.
        self.project_repo.delete(id).await?;
        self.policy_control.reload().await
    }

    #[instrument(skip(self, caller))]
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

    #[instrument(skip(self, caller), fields(organization_id = %organization_id))]
    pub async fn list_by_organization(
        &self,
        caller: &CallerContext,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        self.permission_service
            .check(
                caller,
                Permission::ListProjectsByOrganization(organization_id.clone()),
            )
            .await?;
        self.project_repo
            .list_by_organization(organization_id, pagination)
            .await
    }

    #[instrument(skip(self, caller), fields(project_id = %project_id))]
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
            .user_project_repo
            .list_members(project_id, pagination)
            .await?;
        let (user_ids, metadata) = paginated.into_parts();

        // Batched read + re-order to the paginated membership order.
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

    #[instrument(skip(self, caller), fields(user_id = %user_id))]
    pub async fn list_user_projects(
        &self,
        caller: &CallerContext,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<Project>, PaginationMetadata)> {
        self.permission_service
            .check(caller, Permission::ListUserProjects(user_id.clone()))
            .await?;

        let paginated = self
            .user_project_repo
            .list_user_projects(user_id, pagination)
            .await?;
        let (project_ids, metadata) = paginated.into_parts();

        let mut by_id: std::collections::HashMap<String, Project> = self
            .project_repo
            .find_by_ids(&project_ids)
            .await?
            .into_iter()
            .map(|p| (p.id().as_str().to_owned(), p))
            .collect();
        let projects = project_ids
            .iter()
            .filter_map(|id| by_id.remove(id.as_str()))
            .collect();

        Ok((projects, metadata))
    }

    #[instrument(skip(self, caller), fields(user_id = %user_id, project_id = %project_id))]
    pub async fn add_user(
        &self,
        caller: &CallerContext,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::AddProjectMember(project_id.clone()))
            .await?;

        if self
            .user_project_repo
            .is_member(user_id, project_id)
            .await?
        {
            return Err(DomainError::conflict(
                "User is already a member of this project",
            ));
        }

        self.user_project_repo.add_member(user_id, project_id).await
    }

    #[instrument(skip(self, caller), fields(user_id = %user_id, project_id = %project_id))]
    pub async fn remove_user(
        &self,
        caller: &CallerContext,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::RemoveProjectMember(project_id.clone()))
            .await?;

        // Dropping the membership row is what cuts the user's authority here:
        // every project-scoped permit is gated on live project membership. The
        // grants go with it — atomically — so re-adding the user later starts
        // from a clean slate instead of silently restoring their old authority.
        // Guard the scope's last human owner before stripping anyone.
        let scope = Scope::Project(project_id.clone());
        let principal = Principal::User(user_id.clone());
        let grants = self.grant_repo.list_all().await?;
        if removal_orphans_scope(&grants, &scope, &principal) {
            return Err(DomainError::business_rule(
                "cannot remove the last owner of this project",
            ));
        }

        self.project_repo
            .remove_member_and_grants(user_id, project_id)
            .await?;
        // The membership gate already denies the ex-member; deleted grant rows
        // additionally leave the live policy set on rebuild.
        self.policy_control.reload().await
    }
}
