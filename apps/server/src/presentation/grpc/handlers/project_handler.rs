use crate::application::dto::{
    AddUserToProjectRequestDto, CreateProjectRequestDto, DeleteProjectRequestDto,
    GetProjectRequestDto, ListProjectUsersRequestDto, ListProjectsRequestDto,
    ListUserProjectsRequestDto, RemoveUserFromProjectRequestDto, ToggleProjectActiveRequestDto,
    UpdateProjectRequestDto,
};
use crate::domain::value_objects::{
    Description, OrganizationId, ProjectId, ProjectName, UserId, UserProjectRole,
};
use crate::presentation::grpc::mappers::{domain_to_proto_metadata, proto_to_domain_pagination};
use crate::presentation::grpc::middleware::check_permissions;
use crate::shared::di::AppContainer;
use derive_more::Constructor;
use protocol::services::project::{
    AddUserToProjectRequest, AddUserToProjectResponse, CreateProjectRequest, DeleteProjectRequest,
    DeleteProjectResponse, GetProjectRequest, ListProjectUsersRequest, ListProjectUsersResponse,
    ListProjectsRequest, ListProjectsResponse, ListUserProjectsRequest, ProjectResponse,
    RemoveUserFromProjectRequest, RemoveUserFromProjectResponse, ToggleProjectActiveRequest,
    ToggleProjectActiveResponse, UpdateProjectRequest, project_service_server,
};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

#[derive(Constructor)]
pub struct ProjectHandler {
    container: Arc<AppContainer>,
}

#[async_trait::async_trait]
impl project_service_server::ProjectService for ProjectHandler {
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        // Get organization_id from request to use as domain for RBAC
        // let org_id = &request.get_ref().organization_id;

        // Check RBAC permissions (token already validated by interceptor)
        let auth_ctx = check_permissions(
            &request,
            self.container.rbac_enforcer(),
            "*", // should be 'org_id' !!!!
            "projects",
            "create",
        )
        .await?;

        let req = request.into_inner();
        let dto = CreateProjectRequestDto {
            name: ProjectName::new(req.name)?,
            description: req.description.map(|s| Description::new(s)).transpose()?,
            organization_id: OrganizationId::new(req.organization_id),
            creator_id: auth_ctx.user_id,
        };

        let response = self
            .container
            .create_project_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        // let project_id = &request.get_ref().project_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     project_id,
        //     "projects",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = GetProjectRequestDto {
            project_id: ProjectId::new(req.project_id),
        };

        let response = self.container.get_project_use_case().execute(dto).await?;

        Ok(Response::new(response.into()))
    }

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        // let project_id = &request.get_ref().project_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     project_id,
        //     "projects",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = UpdateProjectRequestDto {
            project_id: ProjectId::new(req.project_id),
            name: req
                .name
                .filter(|s| !s.is_empty())
                .map(|s| ProjectName::new(s))
                .transpose()?,
            description: req
                .description
                .filter(|s| !s.is_empty())
                .map(Description::new)
                .transpose()?,
        };

        let response = self
            .container
            .update_project_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn toggle_project_active(
        &self,
        request: Request<ToggleProjectActiveRequest>,
    ) -> Result<Response<ToggleProjectActiveResponse>, Status> {
        // let project_id = &request.get_ref().project_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     project_id,
        //     "projects",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = ToggleProjectActiveRequestDto {
            project_id: ProjectId::new(req.project_id),
        };

        let _response = self
            .container
            .toggle_project_active_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(ToggleProjectActiveResponse::default()))
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<DeleteProjectResponse>, Status> {
        // let project_id = &request.get_ref().project_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     project_id,
        //     "projects",
        //     "delete",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = DeleteProjectRequestDto {
            project_id: ProjectId::new(req.project_id),
        };

        let _response = self
            .container
            .delete_project_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(DeleteProjectResponse::default()))
    }

    async fn list_projects(
        &self,
        request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        // Check RBAC permissions for listing all projects
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     "*",
        //     "projects",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let response = self
            .container
            .list_projects_use_case()
            .execute(ListProjectsRequestDto { pagination })
            .await?;

        let projects = response.projects.into_iter().map(Into::into).collect();
        let pagination = response.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListProjectsResponse {
            projects,
            pagination,
        }))
    }

    async fn list_project_users(
        &self,
        request: Request<ListProjectUsersRequest>,
    ) -> Result<Response<ListProjectUsersResponse>, Status> {
        // let project_id = &request.get_ref().project_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     project_id,
        //     "projects",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let dto = ListProjectUsersRequestDto {
            project_id: ProjectId::new(req.project_id),
            pagination,
        };

        let response = self
            .container
            .list_project_users_use_case()
            .execute(dto)
            .await?;

        let users = response.users.into_iter().map(Into::into).collect();
        let pagination = response.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListProjectUsersResponse {
            users,
            pagination,
        }))
    }

    async fn list_user_projects(
        &self,
        request: Request<ListUserProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        // Check RBAC permissions for listing user projects
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     "*",
        //     "projects",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let dto = ListUserProjectsRequestDto {
            user_id: UserId::new(req.user_id),
            pagination,
        };

        let response = self
            .container
            .list_user_projects_use_case()
            .execute(dto)
            .await?;

        let projects = response.projects.into_iter().map(Into::into).collect();
        let pagination = response.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListProjectsResponse {
            projects,
            pagination,
        }))
    }

    async fn add_user_to_project(
        &self,
        request: Request<AddUserToProjectRequest>,
    ) -> Result<Response<AddUserToProjectResponse>, Status> {
        // let project_id = &request.get_ref().project_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     project_id,
        //     "projects",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = AddUserToProjectRequestDto {
            user_id: UserId::new(req.user_id),
            project_id: ProjectId::new(req.project_id),
            role: UserProjectRole::new(req.role)?,
        };

        let response = self
            .container
            .add_user_to_project_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(AddUserToProjectResponse {
            relation_id: response.relation_id.to_string(),
        }))
    }

    async fn remove_user_from_project(
        &self,
        request: Request<RemoveUserFromProjectRequest>,
    ) -> Result<Response<RemoveUserFromProjectResponse>, Status> {
        // let project_id = &request.get_ref().project_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     project_id,
        //     "projects",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = RemoveUserFromProjectRequestDto {
            user_id: UserId::new(req.user_id),
            project_id: ProjectId::new(req.project_id),
        };

        let _response = self
            .container
            .remove_user_from_project_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(RemoveUserFromProjectResponse::default()))
    }
}
