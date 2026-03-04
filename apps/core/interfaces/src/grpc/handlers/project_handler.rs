use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, project_to_proto, proto_to_domain_pagination,
};
use application::ProjectUseCases;
use derive_more::Constructor;
use domain::entities::{OrganizationId, ProjectId, UserId};
use domain::ports::{PermissionService, ProjectRepository, UserProjectRepository, UserRepository};
use domain::value_objects::permission::policy;
use domain::value_objects::project::{ProjectDescription, ProjectName};
use protocol::services::project::{
    AddUserToProjectRequest, AddUserToProjectResponse, CreateProjectRequest, DeleteProjectRequest,
    DeleteProjectResponse, GetProjectRequest, ListProjectUsersRequest, ListProjectUsersResponse,
    ListProjectsRequest, ListProjectsResponse, ListUserProjectsRequest, ProjectResponse,
    ProjectUserInfoResponse, RemoveUserFromProjectRequest, RemoveUserFromProjectResponse,
    ToggleProjectActiveRequest, ToggleProjectActiveResponse, UpdateProjectRequest,
    project_service_server::ProjectService,
};
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
