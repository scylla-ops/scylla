//! The adapter: each RPC is one `send` or `query` and its response. Parsing lives in the
//! project mapper, behind `Parse`; no RPC checks a permission or touches a port.

use crate::application::{ProjectRepository, ProjectUseCases, UserRepository};
use crate::grpc::adapter::{query, send};
use crate::grpc::mappers::project_to_proto;
use derive_more::Constructor;
use scylla_auth::authz::{PermissionService, PolicyControl};
use scylla_extension::Actions;
use scylla_proto::project::v1::{
    CreateProjectRequest, CreateProjectResponse, DeleteProjectRequest, DeleteProjectResponse,
    GetProjectRequest, GetProjectResponse, ListOrganizationProjectsRequest,
    ListOrganizationProjectsResponse, ListProjectMembersRequest, ListProjectMembersResponse,
    ListProjectsRequest, ListProjectsResponse, ListUserProjectsRequest, ListUserProjectsResponse,
    SetProjectActiveRequest, SetProjectActiveResponse, UpdateProjectRequest, UpdateProjectResponse,
    project_service_server::ProjectService,
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
    actions: Arc<Actions>,
    projects: Arc<ProjectUseCases<P, U, PS, PC>>,
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
        let project = send(&self.actions, &*self.projects, request).await?;
        Ok(Response::new(CreateProjectResponse {
            project: Some(project_to_proto(&project)),
        }))
    }

    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<GetProjectResponse>, Status> {
        let project = query(&self.actions, &*self.projects, request).await?;
        Ok(Response::new(GetProjectResponse {
            project: Some(project_to_proto(&project)),
        }))
    }

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<UpdateProjectResponse>, Status> {
        let project = send(&self.actions, &*self.projects, request).await?;
        Ok(Response::new(UpdateProjectResponse {
            project: Some(project_to_proto(&project)),
        }))
    }

    async fn set_project_active(
        &self,
        request: Request<SetProjectActiveRequest>,
    ) -> Result<Response<SetProjectActiveResponse>, Status> {
        let project = send(&self.actions, &*self.projects, request).await?;
        Ok(Response::new(SetProjectActiveResponse {
            project: Some(project_to_proto(&project)),
        }))
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<DeleteProjectResponse>, Status> {
        send(&self.actions, &*self.projects, request).await?;
        Ok(Response::new(DeleteProjectResponse {}))
    }

    async fn list_projects(
        &self,
        request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let page = query(&self.actions, &*self.projects, request).await?;
        Ok(Response::new(page.into()))
    }

    async fn list_organization_projects(
        &self,
        request: Request<ListOrganizationProjectsRequest>,
    ) -> Result<Response<ListOrganizationProjectsResponse>, Status> {
        let page = query(&self.actions, &*self.projects, request).await?;
        Ok(Response::new(page.into()))
    }

    async fn list_project_members(
        &self,
        request: Request<ListProjectMembersRequest>,
    ) -> Result<Response<ListProjectMembersResponse>, Status> {
        let page = query(&self.actions, &*self.projects, request).await?;
        Ok(Response::new(page.into()))
    }

    async fn list_user_projects(
        &self,
        request: Request<ListUserProjectsRequest>,
    ) -> Result<Response<ListUserProjectsResponse>, Status> {
        let page = query(&self.actions, &*self.projects, request).await?;
        Ok(Response::new(page.into()))
    }
}
