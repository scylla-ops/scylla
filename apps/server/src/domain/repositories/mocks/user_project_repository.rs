//! Mock implementation for UserProjectRepository
//!
//! Provides mockall-based mocks for testing use cases that depend on UserProjectRepository.

use crate::domain::entities::UserProject;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::UserProjectRepository as UserProjRepoTrait;
use crate::domain::value_objects::{
    PaginatedResult, PaginationParams, ProjectId, UserId, UserProjectId,
};
use mockall::mock;

/// Simplified trait for mocking (to work around mockall lifetime limitations)
#[async_trait::async_trait]
pub trait UserProjectRepositoryMock: Send + Sync {
    async fn create(&self, user_project: &UserProject) -> DomainResult<UserProject>;
    async fn find_by_id(&self, id: &UserProjectId) -> DomainResult<UserProject>;
    async fn find_by_user_and_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<UserProject>;
    async fn update(&self, user_project: &UserProject) -> DomainResult<UserProject>;
    async fn delete(&self, id: &UserProjectId) -> DomainResult<()>;
    async fn list_all(&self) -> DomainResult<Vec<UserProject>>;
    async fn add_user_to_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        role: &str,
    ) -> DomainResult<UserProjectId>;
    async fn remove_user_from_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<()>;
}

mock! {
    pub UserProjectRepository {}

    #[async_trait::async_trait]
    impl UserProjectRepositoryMock for UserProjectRepository {
        async fn create(&self, user_project: &UserProject) -> DomainResult<UserProject>;
        async fn find_by_id(&self, id: &UserProjectId) -> DomainResult<UserProject>;
        async fn find_by_user_and_project(
            &self,
            user_id: &UserId,
            project_id: &ProjectId,
        ) -> DomainResult<UserProject>;
        async fn update(&self, user_project: &UserProject) -> DomainResult<UserProject>;
        async fn delete(&self, id: &UserProjectId) -> DomainResult<()>;
        async fn list_all(&self) -> DomainResult<Vec<UserProject>>;
        async fn add_user_to_project(
            &self,
            user_id: &UserId,
            project_id: &ProjectId,
            role: &str,
        ) -> DomainResult<UserProjectId>;
        async fn remove_user_from_project(
            &self,
            user_id: &UserId,
            project_id: &ProjectId,
        ) -> DomainResult<()>;
    }
}

/// Adapter to make MockUserProjectRepository work with the actual UserProjectRepository trait
pub struct MockUserProjectRepositoryAdapter {
    pub inner: MockUserProjectRepository,
}

#[async_trait::async_trait]
impl UserProjRepoTrait for MockUserProjectRepositoryAdapter {
    async fn create(&self, user_project: &UserProject) -> DomainResult<UserProject> {
        self.inner.create(user_project).await
    }

    async fn find_by_id(&self, id: &UserProjectId) -> DomainResult<UserProject> {
        self.inner.find_by_id(id).await
    }

    async fn find_by_user_and_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<UserProject> {
        self.inner
            .find_by_user_and_project(user_id, project_id)
            .await
    }

    async fn update(&self, user_project: &UserProject) -> DomainResult<UserProject> {
        self.inner.update(user_project).await
    }

    async fn delete(&self, id: &UserProjectId) -> DomainResult<()> {
        self.inner.delete(id).await
    }

    async fn list_all(&self) -> DomainResult<Vec<UserProject>> {
        self.inner.list_all().await
    }

    async fn list_projects_for_user(
        &self,
        _user_id: &UserId,
        _pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<ProjectId>> {
        unimplemented!("list_projects_for_user not commonly needed in use case tests")
    }

    async fn list_users_in_project(
        &self,
        _project_id: &ProjectId,
        _pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        unimplemented!("list_users_in_project not commonly needed in use case tests")
    }

    async fn add_user_to_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        role: &str,
    ) -> DomainResult<UserProjectId> {
        self.inner
            .add_user_to_project(user_id, project_id, role)
            .await
    }

    async fn remove_user_from_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<()> {
        self.inner
            .remove_user_from_project(user_id, project_id)
            .await
    }
}
