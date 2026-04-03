use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, project_to_proto, proto_to_domain_pagination,
};
use derive_more::Constructor;
use protocol::services::project::{
    AddUserToProjectRequest, AddUserToProjectResponse, CreateProjectRequest, DeleteProjectRequest,
    DeleteProjectResponse, GetProjectRequest, ListProjectUsersRequest, ListProjectUsersResponse,
    ListProjectsRequest, ListProjectsResponse, ListUserProjectsRequest, ProjectResponse,
    ProjectUserInfoResponse, RemoveUserFromProjectRequest, RemoveUserFromProjectResponse,
    ToggleProjectActiveRequest, ToggleProjectActiveResponse, UpdateProjectRequest,
    project_service_server::ProjectService,
};
use scylla_core::application::ProjectUseCases;
use scylla_core::application::ports::{
    PermissionService, ProjectRepository, UserProjectRepository, UserRepository,
};
use scylla_core::domain::entities::{OrganizationId, ProjectId, UserId};
use scylla_core::domain::value_objects::permission::policy;
use scylla_core::domain::value_objects::project::{ProjectDescription, ProjectName};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct ProjectHandler<
    P: ProjectRepository,
    UP: UserProjectRepository,
    U: UserRepository,
    PS: PermissionService,
> {
    use_cases: Arc<ProjectUseCases<P, UP, U>>,
    permission_checker: Arc<PS>,
}

#[async_trait::async_trait]
impl<
    P: ProjectRepository + Send + Sync + 'static,
    UP: UserProjectRepository + Send + Sync + 'static,
    U: UserRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> ProjectService for ProjectHandler<P, UP, U, PS>
{
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        let target_orga_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(self, request, policy::project::create(target_orga_id));
        let req = request.into_inner();

        let name = ProjectName::new(&req.name).map_err(domain_error_to_status)?;
        let description = req
            .description
            .map(|d| ProjectDescription::new(&d))
            .transpose()
            .map_err(domain_error_to_status)?;
        let organization_id = OrganizationId::new(&req.organization_id);

        let project = self
            .use_cases
            .create(name, description, organization_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(project_to_proto(&project)))
    }

    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        let target_project_id = ProjectId::new(&request.get_ref().project_id);
        require_permission!(self, request, policy::project::get(target_project_id));

        let req = request.into_inner();
        let id = ProjectId::new(&req.project_id);

        let project = self
            .use_cases
            .get(&id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(project_to_proto(&project)))
    }

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        let target_project_id = ProjectId::new(&request.get_ref().project_id);
        require_permission!(self, request, policy::project::update(target_project_id));

        let req = request.into_inner();
        let id = ProjectId::new(&req.project_id);

        let name = req
            .name
            .map(|n| ProjectName::new(&n))
            .transpose()
            .map_err(domain_error_to_status)?;
        let description = req
            .description
            .map(|d| ProjectDescription::new(&d).map(Some))
            .transpose()
            .map_err(domain_error_to_status)?;

        let project = self
            .use_cases
            .update(&id, name, description)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(project_to_proto(&project)))
    }

    async fn toggle_project_active(
        &self,
        request: Request<ToggleProjectActiveRequest>,
    ) -> Result<Response<ToggleProjectActiveResponse>, Status> {
        let target_project_id = ProjectId::new(&request.get_ref().project_id);
        require_permission!(
            self,
            request,
            policy::project::toggle_active(target_project_id)
        );

        let req = request.into_inner();
        let id = ProjectId::new(&req.project_id);

        self.use_cases
            .toggle_active(&id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ToggleProjectActiveResponse {}))
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<DeleteProjectResponse>, Status> {
        let target_project_id = ProjectId::new(&request.get_ref().project_id);
        require_permission!(self, request, policy::project::delete(target_project_id));

        let req = request.into_inner();
        let id = ProjectId::new(&req.project_id);

        self.use_cases
            .delete(&id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteProjectResponse {}))
    }

    async fn list_projects(
        &self,
        request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        require_permission!(self, request, policy::project::list());

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list(pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (projects, metadata) = result.into_parts();
        let projects: Vec<ProjectResponse> = projects.iter().map(project_to_proto).collect();

        Ok(Response::new(ListProjectsResponse {
            projects,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_project_users(
        &self,
        request: Request<ListProjectUsersRequest>,
    ) -> Result<Response<ListProjectUsersResponse>, Status> {
        let target_project_id = ProjectId::new(&request.get_ref().project_id);
        require_permission!(
            self,
            request,
            policy::project::list_users(target_project_id)
        );

        let req = request.into_inner();
        let project_id = ProjectId::new(&req.project_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (users, metadata) = self
            .use_cases
            .list_users(&project_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let users = users
            .iter()
            .map(|user| ProjectUserInfoResponse {
                user_id: user.id().to_string(),
                username: user.username().to_string(),
            })
            .collect();

        Ok(Response::new(ListProjectUsersResponse {
            users,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_user_projects(
        &self,
        request: Request<ListUserProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let target_user_id = UserId::new(&request.get_ref().user_id);
        require_permission!(
            self,
            request,
            policy::project::list_user_projects(target_user_id)
        );

        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (projects, metadata) = self
            .use_cases
            .list_user_projects(&user_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let projects: Vec<ProjectResponse> = projects.iter().map(project_to_proto).collect();

        Ok(Response::new(ListProjectsResponse {
            projects,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn add_user_to_project(
        &self,
        request: Request<AddUserToProjectRequest>,
    ) -> Result<Response<AddUserToProjectResponse>, Status> {
        let target_project_id = ProjectId::new(&request.get_ref().project_id);
        require_permission!(
            self,
            request,
            policy::project::add_user_to_project(target_project_id)
        );

        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let project_id = ProjectId::new(&req.project_id);

        self.use_cases
            .add_user(&user_id, &project_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(AddUserToProjectResponse {}))
    }

    async fn remove_user_from_project(
        &self,
        request: Request<RemoveUserFromProjectRequest>,
    ) -> Result<Response<RemoveUserFromProjectResponse>, Status> {
        let target_project_id = ProjectId::new(&request.get_ref().project_id);
        require_permission!(
            self,
            request,
            policy::project::remove_user_from_project(target_project_id)
        );

        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let project_id = ProjectId::new(&req.project_id);

        self.use_cases
            .remove_user(&user_id, &project_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RemoveUserFromProjectResponse {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_interceptor::AuthContext;
    use protocol::services::project::project_service_server::ProjectService;
    use scylla_core::application::ProjectUseCases;
    use scylla_core::application::ports::{ProjectRepository, UserProjectRepository, UserRepository};
    use scylla_core::application::ports::services::permission_service::PermissionService;
    use scylla_core::domain::entities::{EntityId, Project, User};
    use scylla_core::domain::errors::DomainResult;
    use scylla_core::domain::value_objects::project::{ProjectDescription, ProjectName};
    use scylla_core::domain::value_objects::user::Username;
    use scylla_core::domain::value_objects::permission::policy::{GroupingPolicy, Policy};
    use scylla_core::domain::value_objects::{PaginatedResult, PaginationParams};
    use async_trait::async_trait;
    use std::sync::Arc;

    // ── Stubs ──────────────────────────────────────────────────

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
        async fn create(&self, p: &Project) -> DomainResult<Project> { (self.create_fn.as_ref().unwrap())(p) }
        async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> { (self.find_by_id_fn.as_ref().unwrap())(id) }
        async fn update(&self, p: &Project) -> DomainResult<Project> { (self.update_fn.as_ref().unwrap())(p) }
        async fn delete(&self, id: &ProjectId) -> DomainResult<()> { (self.delete_fn.as_ref().unwrap())(id) }
        async fn list_all(&self, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Project>> { (self.list_all_fn.as_ref().unwrap())() }
        async fn list_active(&self, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Project>> { unimplemented!() }
    }

    #[derive(Default)]
    struct StubUserProjectRepo {
        add_member_fn: Option<Box<dyn Fn(&UserId, &ProjectId) -> DomainResult<()> + Send + Sync>>,
        remove_member_fn: Option<Box<dyn Fn(&UserId, &ProjectId) -> DomainResult<()> + Send + Sync>>,
        is_member_fn: Option<Box<dyn Fn(&UserId, &ProjectId) -> DomainResult<bool> + Send + Sync>>,
    }

    #[async_trait]
    impl UserProjectRepository for StubUserProjectRepo {
        async fn add_member(&self, uid: &UserId, pid: &ProjectId) -> DomainResult<()> { (self.add_member_fn.as_ref().unwrap())(uid, pid) }
        async fn remove_member(&self, uid: &UserId, pid: &ProjectId) -> DomainResult<()> { (self.remove_member_fn.as_ref().unwrap())(uid, pid) }
        async fn is_member(&self, uid: &UserId, pid: &ProjectId) -> DomainResult<bool> { (self.is_member_fn.as_ref().unwrap())(uid, pid) }
        async fn list_members(&self, _pid: &ProjectId, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<UserId>> { unimplemented!() }
        async fn list_user_projects(&self, _uid: &UserId, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<ProjectId>> { unimplemented!() }
    }

    #[derive(Default)]
    struct StubUserRepo;

    #[async_trait]
    impl UserRepository for StubUserRepo {
        async fn create(&self, _u: &User) -> DomainResult<User> { unimplemented!() }
        async fn find_by_id(&self, _id: &UserId) -> DomainResult<User> { unimplemented!() }
        async fn find_by_username(&self, _u: &Username) -> DomainResult<User> { unimplemented!() }
        async fn update(&self, _u: &User) -> DomainResult<User> { unimplemented!() }
        async fn delete(&self, _id: &UserId) -> DomainResult<()> { unimplemented!() }
        async fn list_all(&self, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<User>> { unimplemented!() }
        async fn username_exists(&self, _u: &Username) -> DomainResult<bool> { unimplemented!() }
    }

    struct AllowAll;

    #[async_trait]
    impl PermissionService for AllowAll {
        async fn check(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> { Ok(true) }
        async fn add_policy(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> { Ok(true) }
        async fn remove_policy(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> { Ok(true) }
        async fn list_policies(&self, _s: Option<&str>) -> DomainResult<Vec<(String, Policy)>> { Ok(vec![]) }
        async fn add_grouping_policy(&self, _s: impl EntityId, _p: GroupingPolicy) -> DomainResult<bool> { Ok(true) }
        async fn remove_grouping_policy(&self, _s: impl EntityId, _p: GroupingPolicy) -> DomainResult<bool> { Ok(true) }
        async fn list_grouping_policies(&self, _s: Option<&str>) -> DomainResult<Vec<(String, GroupingPolicy)>> { Ok(vec![]) }
    }

    // ── Helpers ─────────────────────────────────────────────────

    fn test_project() -> Project {
        Project::create(
            ProjectName::new("testproj").unwrap(),
            Some(ProjectDescription::new("desc").unwrap()),
            OrganizationId::generate(),
        ).unwrap()
    }

    fn authed_request<T>(body: T) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut().insert(AuthContext::new(UserId::generate()));
        req
    }

    fn make_handler(
        project_repo: StubProjectRepo,
        user_project_repo: StubUserProjectRepo,
        user_repo: StubUserRepo,
    ) -> ProjectHandler<StubProjectRepo, StubUserProjectRepo, StubUserRepo, AllowAll> {
        let uc = Arc::new(ProjectUseCases::new(
            Arc::new(project_repo),
            Arc::new(user_project_repo),
            Arc::new(user_repo),
        ));
        ProjectHandler::new(uc, Arc::new(AllowAll))
    }

    // ── Tests ───────────────────────────────────────────────────

    #[tokio::test]
    async fn create_project_returns_proto() {
        let mut repo = StubProjectRepo::default();
        repo.create_fn = Some(Box::new(|p| Ok(p.clone())));

        let handler = make_handler(repo, StubUserProjectRepo::default(), StubUserRepo);
        let req = authed_request(CreateProjectRequest {
            name: "newproj".into(),
            description: Some("desc".into()),
            organization_id: OrganizationId::generate().to_string(),
        });

        let resp = handler.create_project(req).await.unwrap();
        assert_eq!(resp.into_inner().name, "newproj");
    }

    #[tokio::test]
    async fn get_project_returns_proto() {
        let proj = test_project();
        let proj_id_str = proj.id().to_string();
        let p = proj.clone();

        let mut repo = StubProjectRepo::default();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(p.clone())));

        let handler = make_handler(repo, StubUserProjectRepo::default(), StubUserRepo);
        let req = authed_request(GetProjectRequest { project_id: proj_id_str });

        let resp = handler.get_project(req).await.unwrap();
        assert_eq!(resp.into_inner().name, "testproj");
    }

    #[tokio::test]
    async fn delete_project_success() {
        let proj = test_project();
        let proj_id_str = proj.id().to_string();

        let mut repo = StubProjectRepo::default();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(proj.clone())));
        repo.delete_fn = Some(Box::new(|_| Ok(())));

        let handler = make_handler(repo, StubUserProjectRepo::default(), StubUserRepo);
        let req = authed_request(DeleteProjectRequest { project_id: proj_id_str });
        assert!(handler.delete_project(req).await.is_ok());
    }

    #[tokio::test]
    async fn list_projects_returns_empty() {
        let mut repo = StubProjectRepo::default();
        repo.list_all_fn = Some(Box::new(|| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![], &params, 0))
        }));

        let handler = make_handler(repo, StubUserProjectRepo::default(), StubUserRepo);
        let req = authed_request(ListProjectsRequest { pagination: None });

        let resp = handler.list_projects(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.projects.is_empty());
        assert!(inner.pagination.is_some());
    }

    #[tokio::test]
    async fn add_user_to_project_success() {
        let proj = test_project();
        let proj_id_str = proj.id().to_string();
        let user_id = UserId::generate();

        let mut user_proj_repo = StubUserProjectRepo::default();
        user_proj_repo.is_member_fn = Some(Box::new(|_, _| Ok(false)));
        user_proj_repo.add_member_fn = Some(Box::new(|_, _| Ok(())));

        let handler = make_handler(StubProjectRepo::default(), user_proj_repo, StubUserRepo);
        let req = authed_request(AddUserToProjectRequest {
            project_id: proj_id_str,
            user_id: user_id.to_string(),
        });

        assert!(handler.add_user_to_project(req).await.is_ok());
    }

    #[tokio::test]
    async fn create_project_without_auth_fails() {
        let handler = make_handler(StubProjectRepo::default(), StubUserProjectRepo::default(), StubUserRepo);
        let req = Request::new(CreateProjectRequest {
            name: "proj".into(),
            description: None,
            organization_id: OrganizationId::generate().to_string(),
        });

        let err = handler.create_project(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Internal);
    }
}
