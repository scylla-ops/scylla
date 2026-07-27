use crate::application::authz::grant::{Grant, PROJECT_ADMIN_ROLE, Principal, Scope};
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::{Visibility, VisibilityResolver};
use crate::application::caller::CallerContext;
use crate::application::{PermissionService, ProjectRepository, UserRepository};
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
    U: UserRepository,
    PS: PermissionService,
    PC: PolicyControl,
> {
    project_repo: Arc<P>,
    user_repo: Arc<U>,
    permission_service: Arc<PS>,
    /// Narrows listings to what the caller holds, for the pages a single
    /// yes/no check cannot answer.
    visibility: Arc<dyn VisibilityResolver>,
    policy_control: Arc<PC>,
    /// Per-org limits enforced on project creation.
    quotas: crate::application::quota::Quotas,
}

impl<P: ProjectRepository, U: UserRepository, PS: PermissionService, PC: PolicyControl>
    ProjectUseCases<P, U, PS, PC>
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

        // The human creator becomes the project's admin. The project row and the
        // owner grant are written in ONE transaction, so a partial failure can
        // never leave a project without an owner. Machine/anonymous callers have
        // nobody to make owner, so they just get the bare project.
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

    /// The organization's projects, narrowed to the ones the caller may see.
    ///
    /// Deliberately not gated on `listProjectsByOrganization`: that permission
    /// means "see *all* of them" and comes from an organization-wide role.
    /// Someone holding only a project role has no business being refused the
    /// whole listing — they should see their own project and nothing else,
    /// which is what the filter expresses. Reading the organization at all is
    /// still required, so this is not an open endpoint.
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
                Permission::ReadOrganization(organization_id.clone()),
            )
            .await?;

        // Holding the org-wide list permission means nothing is hidden; anything
        // else is narrowed to the scopes the caller actually holds.
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

    /// Everyone on the project: the principals holding a grant scoped to it.
    /// Holders of an organization-wide grant are not listed — they administer
    /// the organization rather than work here, and the scope hierarchy already
    /// gives them the access they need.
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
            .project_repo
            .list_principals(project_id, pagination)
            .await?;
        let (user_ids, metadata) = paginated.into_parts();

        // Batched read + re-order to the paginated order.
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

    /// The projects a user works on: granted directly, or through a grant on the
    /// owning organization.
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

        let paginated = self.project_repo.list_for_user(user_id, pagination).await?;
        Ok(paginated.into_parts())
    }
}
