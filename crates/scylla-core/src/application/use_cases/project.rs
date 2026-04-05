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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{ProjectRepository, UserProjectRepository, UserRepository};
    use crate::domain::value_objects::project::ProjectName;
    use crate::domain::value_objects::user::Username;
    use async_trait::async_trait;
    use std::sync::Arc;

    #[derive(Default)]
    struct StubProjectRepo {
        create_fn: Option<Box<dyn Fn(&Project) -> DomainResult<Project> + Send + Sync>>,
        find_by_id_fn: Option<Box<dyn Fn(&ProjectId) -> DomainResult<Project> + Send + Sync>>,
        update_fn: Option<Box<dyn Fn(&Project) -> DomainResult<Project> + Send + Sync>>,
        delete_fn: Option<Box<dyn Fn(&ProjectId) -> DomainResult<()> + Send + Sync>>,
        list_all_fn: Option<Box<dyn Fn() -> DomainResult<PaginatedResult<Project>> + Send + Sync>>,
    }

    #[async_trait]
    impl ProjectRepository for StubProjectRepo {
        async fn create(&self, p: &Project) -> DomainResult<Project> {
            (self.create_fn.as_ref().unwrap())(p)
        }
        async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn update(&self, p: &Project) -> DomainResult<Project> {
            (self.update_fn.as_ref().unwrap())(p)
        }
        async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
            (self.delete_fn.as_ref().unwrap())(id)
        }
        async fn list_all(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Project>> {
            (self.list_all_fn.as_ref().unwrap())()
        }
        async fn list_active(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Project>> {
            unimplemented!()
        }
    }

    #[derive(Default)]
    struct StubUserProjectRepo {
        add_member_fn: Option<Box<dyn Fn(&UserId, &ProjectId) -> DomainResult<()> + Send + Sync>>,
        remove_member_fn:
            Option<Box<dyn Fn(&UserId, &ProjectId) -> DomainResult<()> + Send + Sync>>,
        is_member_fn: Option<Box<dyn Fn(&UserId, &ProjectId) -> DomainResult<bool> + Send + Sync>>,
    }

    #[async_trait]
    impl UserProjectRepository for StubUserProjectRepo {
        async fn add_member(&self, uid: &UserId, pid: &ProjectId) -> DomainResult<()> {
            (self.add_member_fn.as_ref().unwrap())(uid, pid)
        }
        async fn remove_member(&self, uid: &UserId, pid: &ProjectId) -> DomainResult<()> {
            (self.remove_member_fn.as_ref().unwrap())(uid, pid)
        }
        async fn is_member(&self, uid: &UserId, pid: &ProjectId) -> DomainResult<bool> {
            (self.is_member_fn.as_ref().unwrap())(uid, pid)
        }
        async fn list_members(
            &self,
            _pid: &ProjectId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<UserId>> {
            unimplemented!()
        }
        async fn list_user_projects(
            &self,
            _uid: &UserId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<ProjectId>> {
            unimplemented!()
        }
    }

    #[derive(Default)]
    struct StubUserRepo;

    #[async_trait]
    impl UserRepository for StubUserRepo {
        async fn create(&self, _u: &User) -> DomainResult<User> {
            unimplemented!()
        }
        async fn find_by_id(&self, _id: &UserId) -> DomainResult<User> {
            unimplemented!()
        }
        async fn find_by_username(&self, _u: &Username) -> DomainResult<User> {
            unimplemented!()
        }
        async fn update(&self, _u: &User) -> DomainResult<User> {
            unimplemented!()
        }
        async fn delete(&self, _id: &UserId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_all(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<User>> {
            unimplemented!()
        }
        async fn username_exists(&self, _u: &Username) -> DomainResult<bool> {
            unimplemented!()
        }
    }

    fn test_project() -> Project {
        Project::create(
            ProjectName::new("Test Project").unwrap(),
            None,
            OrganizationId::generate(),
        )
        .unwrap()
    }

    fn make_uc(
        project_repo: StubProjectRepo,
        user_project_repo: StubUserProjectRepo,
        user_repo: StubUserRepo,
    ) -> ProjectUseCases<StubProjectRepo, StubUserProjectRepo, StubUserRepo> {
        ProjectUseCases::new(
            Arc::new(project_repo),
            Arc::new(user_project_repo),
            Arc::new(user_repo),
        )
    }

    #[tokio::test]
    async fn create_success() {
        let mut repo = StubProjectRepo::default();
        repo.create_fn = Some(Box::new(|p| Ok(p.clone())));

        let uc = make_uc(repo, StubUserProjectRepo::default(), StubUserRepo);
        let name = ProjectName::new("My Project").unwrap();
        let org_id = OrganizationId::generate();
        let project = uc.create(name, None, org_id).await.unwrap();
        assert_eq!(project.name().as_str(), "My Project");
        assert!(project.is_active());
    }

    #[tokio::test]
    async fn get_project() {
        let project = test_project();
        let mut repo = StubProjectRepo::default();
        let p = project.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(p.clone())));

        let uc = make_uc(repo, StubUserProjectRepo::default(), StubUserRepo);
        let result = uc.get(project.id()).await.unwrap();
        assert_eq!(result.name().as_str(), "Test Project");
    }

    #[tokio::test]
    async fn update_name() {
        let project = test_project();
        let mut repo = StubProjectRepo::default();
        let p = project.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(p.clone())));
        repo.update_fn = Some(Box::new(|p| Ok(p.clone())));

        let uc = make_uc(repo, StubUserProjectRepo::default(), StubUserRepo);
        let new_name = ProjectName::new("Updated").unwrap();
        let result = uc.update(project.id(), Some(new_name), None).await.unwrap();
        assert_eq!(result.name().as_str(), "Updated");
    }

    #[tokio::test]
    async fn toggle_active() {
        let project = test_project();
        let mut repo = StubProjectRepo::default();
        let p = project.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(p.clone())));
        repo.update_fn = Some(Box::new(|p| Ok(p.clone())));

        let uc = make_uc(repo, StubUserProjectRepo::default(), StubUserRepo);
        assert!(uc.toggle_active(project.id()).await.is_ok());
    }

    #[tokio::test]
    async fn delete_project() {
        let project = test_project();
        let mut repo = StubProjectRepo::default();
        let p = project.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(p.clone())));
        repo.delete_fn = Some(Box::new(|_| Ok(())));

        let uc = make_uc(repo, StubUserProjectRepo::default(), StubUserRepo);
        assert!(uc.delete(project.id()).await.is_ok());
    }

    #[tokio::test]
    async fn add_user_success() {
        let mut up = StubUserProjectRepo::default();
        up.is_member_fn = Some(Box::new(|_, _| Ok(false)));
        up.add_member_fn = Some(Box::new(|_, _| Ok(())));

        let uc = make_uc(StubProjectRepo::default(), up, StubUserRepo);
        assert!(
            uc.add_user(&UserId::generate(), &ProjectId::generate())
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn add_user_already_member() {
        let mut up = StubUserProjectRepo::default();
        up.is_member_fn = Some(Box::new(|_, _| Ok(true)));

        let uc = make_uc(StubProjectRepo::default(), up, StubUserRepo);
        let result = uc
            .add_user(&UserId::generate(), &ProjectId::generate())
            .await;
        assert!(matches!(result.unwrap_err(), DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn remove_user_success() {
        let mut up = StubUserProjectRepo::default();
        up.remove_member_fn = Some(Box::new(|_, _| Ok(())));

        let uc = make_uc(StubProjectRepo::default(), up, StubUserRepo);
        assert!(
            uc.remove_user(&UserId::generate(), &ProjectId::generate())
                .await
                .is_ok()
        );
    }
}
