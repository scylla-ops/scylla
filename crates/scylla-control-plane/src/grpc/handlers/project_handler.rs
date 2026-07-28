use crate::extract_auth_context;
use crate::grpc::convert::{required, wrap};
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, project_to_proto, proto_to_domain_pagination,
};
use derive_more::Constructor;
use scylla_core::application::ProjectUseCases;
use scylla_core::application::authz::policy::PolicyControl;
use scylla_core::application::{PermissionService, ProjectRepository, UserRepository};
use scylla_core::domain::entities::{OrganizationId, ProjectId, UserId};
use scylla_core::domain::value_objects::project::{ProjectDescription, ProjectName};
use scylla_protocol::project::v1::{
    CreateProjectRequest, CreateProjectResponse, DeleteProjectRequest, DeleteProjectResponse,
    GetProjectRequest, GetProjectResponse, ListOrganizationProjectsRequest,
    ListOrganizationProjectsResponse, ListProjectMembersRequest, ListProjectMembersResponse,
    ListProjectsRequest, ListProjectsResponse, ListUserProjectsRequest, ListUserProjectsResponse,
    Project, ProjectMember, SetProjectActiveRequest, SetProjectActiveResponse,
    UpdateProjectRequest, UpdateProjectResponse, project_service_server::ProjectService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct ProjectHandler<
    P: ProjectRepository,
    U: UserRepository,
    PS: PermissionService,
    PC: PolicyControl,
> {
    use_cases: Arc<ProjectUseCases<P, U, PS, PC>>,
}

#[async_trait::async_trait]
impl<
    P: ProjectRepository + Send + Sync + 'static,
    U: UserRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
> ProjectService for ProjectHandler<P, U, PS, PC>
{
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let name = ProjectName::new(&req.name).map_err(domain_error_to_status)?;
        let description = req
            .description
            .map(|d| ProjectDescription::new(&d))
            .transpose()
            .map_err(domain_error_to_status)?;
        let organization_id =
            OrganizationId::new(&required(req.organization_id, "organization_id")?);

        let project = self
            .use_cases
            .create(&caller, name, description, organization_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(CreateProjectResponse {
            project: Some(project_to_proto(&project)),
        }))
    }

    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<GetProjectResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = ProjectId::new(&required(req.project_id, "project_id")?);

        let project = self
            .use_cases
            .get(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(GetProjectResponse {
            project: Some(project_to_proto(&project)),
        }))
    }

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<UpdateProjectResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = ProjectId::new(&required(req.project_id, "project_id")?);

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

        Ok(Response::new(UpdateProjectResponse {
            project: Some(project_to_proto(&project)),
        }))
    }

    /// Sets the active flag to the requested value instead of flipping it, so a
    /// retried call lands on the state the caller asked for.
    async fn set_project_active(
        &self,
        request: Request<SetProjectActiveRequest>,
    ) -> Result<Response<SetProjectActiveResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = ProjectId::new(&required(req.project_id, "project_id")?);

        let project = self
            .use_cases
            .set_active(&caller, &id, req.is_active)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(SetProjectActiveResponse {
            project: Some(project_to_proto(&project)),
        }))
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<DeleteProjectResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = ProjectId::new(&required(req.project_id, "project_id")?);

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
        let projects: Vec<Project> = projects.iter().map(project_to_proto).collect();

        Ok(Response::new(ListProjectsResponse {
            projects,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_organization_projects(
        &self,
        request: Request<ListOrganizationProjectsRequest>,
    ) -> Result<Response<ListOrganizationProjectsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let organization_id =
            OrganizationId::new(&required(req.organization_id, "organization_id")?);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_organization(&caller, &organization_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (projects, metadata) = result.into_parts();
        let projects: Vec<Project> = projects.iter().map(project_to_proto).collect();

        Ok(Response::new(ListOrganizationProjectsResponse {
            projects,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_project_members(
        &self,
        request: Request<ListProjectMembersRequest>,
    ) -> Result<Response<ListProjectMembersResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let project_id = ProjectId::new(&required(req.project_id, "project_id")?);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (users, metadata) = self
            .use_cases
            .list_users(&caller, &project_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let members = users
            .iter()
            .map(|user| ProjectMember {
                user_id: wrap(user.id().to_string()),
                username: user.username().to_string(),
            })
            .collect();

        Ok(Response::new(ListProjectMembersResponse {
            members,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_user_projects(
        &self,
        request: Request<ListUserProjectsRequest>,
    ) -> Result<Response<ListUserProjectsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let user_id = UserId::new(&required(req.user_id, "user_id")?);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (projects, metadata) = self
            .use_cases
            .list_user_projects(&caller, &user_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let projects: Vec<Project> = projects.iter().map(project_to_proto).collect();

        Ok(Response::new(ListUserProjectsResponse {
            projects,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }
}
