use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, project_to_proto, proto_to_domain_pagination,
};
use derive_more::Constructor;
use scylla_core::application::ProjectUseCases;
use scylla_core::application::authz::policy::PolicyControl;
use scylla_core::application::{
    PermissionService, ProjectRepository, UserProjectRepository, UserRepository,
};
use scylla_core::domain::entities::{OrganizationId, ProjectId, UserId};
use scylla_core::domain::value_objects::project::{ProjectDescription, ProjectName};
use scylla_protocol::services::project::{
    AddUserToProjectRequest, AddUserToProjectResponse, CreateProjectRequest, DeleteProjectRequest,
    DeleteProjectResponse, GetProjectRequest, ListOrganizationProjectsRequest,
    ListProjectUsersRequest, ListProjectUsersResponse, ListProjectsRequest, ListProjectsResponse,
    ListUserProjectsRequest, ProjectResponse, ProjectUserInfoResponse,
    RemoveUserFromProjectRequest, RemoveUserFromProjectResponse, ToggleProjectActiveRequest,
    ToggleProjectActiveResponse, UpdateProjectRequest, project_service_server::ProjectService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct ProjectHandler<
    P: ProjectRepository,
    UP: UserProjectRepository,
    U: UserRepository,
    PS: PermissionService,
    PC: PolicyControl,
> {
    use_cases: Arc<ProjectUseCases<P, UP, U, PS, PC>>,
}

#[async_trait::async_trait]
impl<
    P: ProjectRepository + Send + Sync + 'static,
    UP: UserProjectRepository + Send + Sync + 'static,
    U: UserRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
> ProjectService for ProjectHandler<P, UP, U, PS, PC>
{
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        let caller = caller!(request);
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
            .create(&caller, name, description, organization_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(project_to_proto(&project)))
    }

    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = ProjectId::new(&req.project_id);

        let project = self
            .use_cases
            .get(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(project_to_proto(&project)))
    }

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        let caller = caller!(request);
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
            .update(&caller, &id, name, description)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(project_to_proto(&project)))
    }

    async fn toggle_project_active(
        &self,
        request: Request<ToggleProjectActiveRequest>,
    ) -> Result<Response<ToggleProjectActiveResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = ProjectId::new(&req.project_id);

        self.use_cases
            .toggle_active(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ToggleProjectActiveResponse {}))
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<DeleteProjectResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = ProjectId::new(&req.project_id);

        self.use_cases
            .delete(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteProjectResponse {}))
    }

    async fn list_projects(
        &self,
        request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list(&caller, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (projects, metadata) = result.into_parts();
        let projects: Vec<ProjectResponse> = projects.iter().map(project_to_proto).collect();

        Ok(Response::new(ListProjectsResponse {
            projects,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_organization_projects(
        &self,
        request: Request<ListOrganizationProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let organization_id = OrganizationId::new(&req.organization_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_organization(&caller, &organization_id, pagination.as_ref())
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
        let caller = caller!(request);
        let req = request.into_inner();
        let project_id = ProjectId::new(&req.project_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (users, metadata) = self
            .use_cases
            .list_users(&caller, &project_id, pagination.as_ref())
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
        let caller = caller!(request);
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (projects, metadata) = self
            .use_cases
            .list_user_projects(&caller, &user_id, pagination.as_ref())
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
        let caller = caller!(request);
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let project_id = ProjectId::new(&req.project_id);

        self.use_cases
            .add_user(&caller, &user_id, &project_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(AddUserToProjectResponse {}))
    }

    async fn remove_user_from_project(
        &self,
        request: Request<RemoveUserFromProjectRequest>,
    ) -> Result<Response<RemoveUserFromProjectResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let project_id = ProjectId::new(&req.project_id);

        self.use_cases
            .remove_user(&caller, &user_id, &project_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RemoveUserFromProjectResponse {}))
    }
}
