use derive_more::Constructor;
use domain::entities::{
    OrganizationId, Project, ProjectId, User, UserId, UserProject, UserProjectId,
};
use domain::errors::{DomainError, DomainResult};
use domain::ports::{ProjectRepository, UserProjectRepository, UserRepository};
use domain::value_objects::project::{ProjectDescription, ProjectName};
use domain::value_objects::{PaginatedResult, PaginationMetadata, PaginationParams};
use std::sync::Arc;

#[derive(Constructor)]
pub struct ProjectUseCases<P: ProjectRepository, UP: UserProjectRepository, U: UserRepository> {
    project_repo: Arc<P>,
    user_project_repo: Arc<UP>,
    user_repo: Arc<U>,
}

impl<P: ProjectRepository, UP: UserProjectRepository, U: UserRepository> ProjectUseCases<P, UP, U> {
    pub async fn create(
        &self,
        name: ProjectName,
        description: Option<ProjectDescription>,
        organization_id: OrganizationId,
    ) -> DomainResult<Project> {
        let project = Project::create(name, description, organization_id)?;
        self.project_repo.create(&project).await
    }

    pub async fn get(&self, id: &ProjectId) -> DomainResult<Project> {
        self.project_repo.find_by_id(id).await
    }

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

    pub async fn toggle_active(&self, id: &ProjectId) -> DomainResult<()> {
        let mut project = self.project_repo.find_by_id(id).await?;
        project.toggle_active()?;
        self.project_repo.update(&project).await?;
        Ok(())
    }

    pub async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        self.project_repo.find_by_id(id).await?;
        self.project_repo.delete(id).await
    }

    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        self.project_repo.list_all(pagination).await
    }

    pub async fn list_users(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<(User, UserProject)>, PaginationMetadata)> {
        let paginated = self
            .user_project_repo
            .list_users_in_project(project_id, pagination)
            .await?;
        let (user_ids, metadata) = paginated.into_parts();

        let mut pairs = Vec::with_capacity(user_ids.len());
        for user_id in &user_ids {
            let user = self.user_repo.find_by_id(user_id).await?;
            let membership = self
                .user_project_repo
                .find_by_user_and_project(user_id, project_id)
                .await?;
            pairs.push((user, membership));
        }

        Ok((pairs, metadata))
    }

    pub async fn list_user_projects(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<Project>, PaginationMetadata)> {
        let paginated = self
            .user_project_repo
            .list_projects_for_user(user_id, pagination)
            .await?;
        let (project_ids, metadata) = paginated.into_parts();

        let mut projects = Vec::with_capacity(project_ids.len());
        for project_id in &project_ids {
            let project = self.project_repo.find_by_id(project_id).await?;
            projects.push(project);
        }

        Ok((projects, metadata))
    }

    pub async fn add_user(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        role: &str,
    ) -> DomainResult<UserProjectId> {
        if self
            .user_project_repo
            .find_by_user_and_project(user_id, project_id)
            .await
            .is_ok()
        {
            return Err(DomainError::conflict(
                "User is already a member of this project",
            ));
        }

        self.user_project_repo
            .add_user_to_project(user_id, project_id, role)
            .await
    }

    pub async fn remove_user(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<()> {
        self.user_project_repo
            .remove_user_from_project(user_id, project_id)
            .await
    }
}
