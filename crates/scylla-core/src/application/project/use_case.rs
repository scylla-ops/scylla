use crate::application::caller::CallerContext;
use crate::application::{
    PermissionService, ProjectRepository, UserProjectRepository, UserRepository,
};
use crate::domain::entities::{OrganizationId, Project, ProjectId, User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::policy;
use crate::domain::value_objects::project::{ProjectDescription, ProjectName};
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
> {
    project_repo: Arc<P>,
    user_project_repo: Arc<UP>,
    user_repo: Arc<U>,
    permission_service: Arc<PS>,
}

impl<
    P: ProjectRepository,
    UP: UserProjectRepository,
    U: UserRepository,
    PS: PermissionService,
> ProjectUseCases<P, UP, U, PS>
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
            .check(caller, policy::project::create(organization_id.clone()))
            .await?;
        let project = Project::create(name, description, organization_id)?;
        self.project_repo.create(&project).await
    }

    #[instrument(skip(self, caller), fields(project_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &ProjectId) -> DomainResult<Project> {
        self.permission_service
            .check(caller, policy::project::get(id.clone()))
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
            .check(caller, policy::project::update(id.clone()))
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

    #[instrument(skip(self, caller), fields(project_id = %id))]
    pub async fn toggle_active(
        &self,
        caller: &CallerContext,
        id: &ProjectId,
    ) -> DomainResult<()> {
        self.permission_service
            .check(caller, policy::project::toggle_active(id.clone()))
            .await?;

        let mut project = self.project_repo.find_by_id(id).await?;
        project.toggle_active()?;
        self.project_repo.update(&project).await?;
        Ok(())
    }

    #[instrument(skip(self, caller), fields(project_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &ProjectId) -> DomainResult<()> {
        self.permission_service
            .check(caller, policy::project::delete(id.clone()))
            .await?;
        self.project_repo.find_by_id(id).await?;
        self.project_repo.delete(id).await
    }

    #[instrument(skip(self, caller))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        self.permission_service
            .check(caller, policy::project::list())
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
                policy::project::list_by_organization(organization_id.clone()),
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
            .check(caller, policy::project::list_users(project_id.clone()))
            .await?;

        let paginated = self
            .user_project_repo
            .list_members(project_id, pagination)
            .await?;
        let (user_ids, metadata) = paginated.into_parts();

        let mut users = Vec::with_capacity(user_ids.len());
        for user_id in &user_ids {
            let user = self.user_repo.find_by_id(user_id).await?;
            users.push(user);
        }

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
            .check(caller, policy::project::list_user_projects(user_id.clone()))
            .await?;

        let paginated = self
            .user_project_repo
            .list_user_projects(user_id, pagination)
            .await?;
        let (project_ids, metadata) = paginated.into_parts();

        let mut projects = Vec::with_capacity(project_ids.len());
        for project_id in &project_ids {
            let project = self.project_repo.find_by_id(project_id).await?;
            projects.push(project);
        }

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
            .check(
                caller,
                policy::project::add_user_to_project(project_id.clone()),
            )
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
            .check(
                caller,
                policy::project::remove_user_from_project(project_id.clone()),
            )
            .await?;
        self.user_project_repo
            .remove_member(user_id, project_id)
            .await
    }
}
