use crate::application::ports::{ProjectRepository, UserProjectRepository, UserRepository};
use crate::domain::entities::{OrganizationId, Project, ProjectId, User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::project::{ProjectDescription, ProjectName};
use crate::domain::value_objects::{PaginatedResult, PaginationMetadata, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct ProjectUseCases<P: ProjectRepository, UP: UserProjectRepository, U: UserRepository> {
    project_repo: Arc<P>,
    user_project_repo: Arc<UP>,
    user_repo: Arc<U>,
}

impl<P: ProjectRepository, UP: UserProjectRepository, U: UserRepository> ProjectUseCases<P, UP, U> {
    #[instrument(skip(self), fields(name = %name, org_id = %organization_id))]
    pub async fn create(
        &self,
        name: ProjectName,
        description: Option<ProjectDescription>,
        organization_id: OrganizationId,
    ) -> DomainResult<Project> {
        let project = Project::create(name, description, organization_id)?;
        self.project_repo.create(&project).await
    }

    #[instrument(skip(self), fields(project_id = %id))]
    pub async fn get(&self, id: &ProjectId) -> DomainResult<Project> {
        self.project_repo.find_by_id(id).await
    }

    #[instrument(skip(self), fields(project_id = %id))]
    pub async fn update(
        &self,
        id: &ProjectId,
        name: Option<ProjectName>,
        description: Option<Option<ProjectDescription>>,
    ) -> DomainResult<Project> {
        let mut project = self.project_repo.find_by_id(id).await?;

        if let Some(new_name) = name {
            project.update_name(new_name)?;
        }
        if let Some(new_desc) = description {
            project.update_description(new_desc)?;
        }

        self.project_repo.update(&project).await
    }

    #[instrument(skip(self), fields(project_id = %id))]
    pub async fn toggle_active(&self, id: &ProjectId) -> DomainResult<()> {
        let mut project = self.project_repo.find_by_id(id).await?;
        project.toggle_active()?;
        self.project_repo.update(&project).await?;
        Ok(())
    }

    #[instrument(skip(self), fields(project_id = %id))]
    pub async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        self.project_repo.find_by_id(id).await?;
        self.project_repo.delete(id).await
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        self.project_repo.list_all(pagination).await
    }

    #[instrument(skip(self), fields(organization_id = %organization_id))]
    pub async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        self.project_repo
            .list_by_organization(organization_id, pagination)
            .await
    }

    #[instrument(skip(self), fields(project_id = %project_id))]
    pub async fn list_users(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<User>, PaginationMetadata)> {
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

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn list_user_projects(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<Project>, PaginationMetadata)> {
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

    #[instrument(skip(self), fields(user_id = %user_id, project_id = %project_id))]
    pub async fn add_user(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<()> {
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

    #[instrument(skip(self), fields(user_id = %user_id, project_id = %project_id))]
    pub async fn remove_user(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<()> {
        self.user_project_repo
            .remove_member(user_id, project_id)
            .await
    }
}

