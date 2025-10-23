use crate::api::grpc::organization::repos::surreal::OrganizationRepositorySurreal;
use crate::api::grpc::project::models::ProjectPatch;
use crate::api::grpc::project::repos::surreal::ProjectRepositorySurreal;
use crate::api::grpc::project::service::{ProjectDomainError, ProjectService};
use crate::api::grpc::rbac::{check_permission, extract_user_from_token, permissions};
use crate::api::grpc::user::repos::surreal::UserRepositorySurreal;
use protocol::services::project::{
    AddUserToProjectRequest, AddUserToProjectResponse, CreateProjectRequest, DeleteProjectRequest,
    DeleteProjectResponse, GetProjectRequest, ListOrganizationProjectsRequest,
    ListProjectUsersRequest, ListProjectUsersResponse, ListProjectsRequest, ListProjectsResponse,
    ListUserProjectsRequest, ProjectResponse, ProjectUserInfo, RemoveUserFromProjectRequest,
    RemoveUserFromProjectResponse, UpdateProjectRequest, project_service_server,
};
use protocol::tonic::{Request, Response, Status};

type ProjRepo = ProjectRepositorySurreal;
type UserRepo = UserRepositorySurreal;
type OrgRepo = OrganizationRepositorySurreal;

pub struct ProjectController;

#[async_trait::async_trait]
impl project_service_server::ProjectService for ProjectController {
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        let user_id = extract_user_from_token(&request).await?;
        let CreateProjectRequest {
            name,
            description,
            organization_id,
        } = request.into_inner();
        let org_id_str = format!("organizations:{}", organization_id);

        // check write permission on organization to create projects
        check_permission(
            &user_id,
            &org_id_str,
            permissions::resources::ORGANIZATIONS,
            permissions::actions::WRITE,
        )
        .await?;

        let org_id = organization_id.into();
        let project = ProjectService::<ProjRepo, UserRepo, OrgRepo>::create_project(
            name,
            description,
            org_id,
        )
        .await
        .map_err(map_err)?;

        Ok(Response::new(ProjectResponse {
            project_id: project.id.key().to_string(),
            name: project.name,
            description: project.description.unwrap_or_default(),
            organization_id: project.organization.key().to_string(),
            is_active: project.is_active,
            created_at: project.created_at.to_rfc3339(),
            updated_at: project.updated_at.to_rfc3339(),
        }))
    }

    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        let user_id = extract_user_from_token(&request).await?;
        let GetProjectRequest { project_id } = request.into_inner();
        let project_id_str = format!("projects:{}", project_id);

        // check read permission on project
        check_permission(
            &user_id,
            &project_id_str,
            permissions::resources::PROJECTS,
            permissions::actions::READ,
        )
        .await?;

        let proj_id = project_id.into();
        let project = ProjectService::<ProjRepo, UserRepo, OrgRepo>::get_project(proj_id)
            .await
            .map_err(map_err)?;
        Ok(Response::new(ProjectResponse {
            project_id: project.id.key().to_string(),
            name: project.name,
            description: project.description.unwrap_or_default(),
            organization_id: project.organization.key().to_string(),
            is_active: project.is_active,
            created_at: project.created_at.to_rfc3339(),
            updated_at: project.updated_at.to_rfc3339(),
        }))
    }

    async fn list_projects(
        &self,
        request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let ListProjectsRequest { page, page_size } = request.into_inner();
        let page_u32 =
            u32::try_from(page).map_err(|_| Status::invalid_argument("page is too big"))?;
        let page_size_u32 = u32::try_from(page_size)
            .map_err(|_| Status::invalid_argument("page_size is too big"))?;
        let (projects, total_count) =
            ProjectService::<ProjRepo, UserRepo, OrgRepo>::list_projects(page_u32, page_size_u32)
                .await
                .map_err(map_err)?;
        Ok(Response::new(ListProjectsResponse {
            total_count: total_count as u64,
            projects: projects
                .into_iter()
                .map(|project| ProjectResponse {
                    project_id: project.id.key().to_string(),
                    name: project.name,
                    description: project.description.unwrap_or_default(),
                    organization_id: project.organization.key().to_string(),
                    is_active: project.is_active,
                    created_at: project.created_at.to_rfc3339(),
                    updated_at: project.updated_at.to_rfc3339(),
                })
                .collect(),
            page,
            page_size,
        }))
    }

    async fn list_organization_projects(
        &self,
        request: Request<ListOrganizationProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let ListOrganizationProjectsRequest {
            organization_id,
            page,
            page_size,
        } = request.into_inner();
        let org_id = organization_id.into();
        let page_u32 =
            u32::try_from(page).map_err(|_| Status::invalid_argument("page is too big"))?;
        let page_size_u32 = u32::try_from(page_size)
            .map_err(|_| Status::invalid_argument("page_size is too big"))?;
        let (projects, total_count) =
            ProjectService::<ProjRepo, UserRepo, OrgRepo>::list_organization_projects(
                org_id,
                page_u32,
                page_size_u32,
            )
            .await
            .map_err(map_err)?;
        Ok(Response::new(ListProjectsResponse {
            total_count: total_count as u64,
            projects: projects
                .into_iter()
                .map(|project| ProjectResponse {
                    project_id: project.id.key().to_string(),
                    name: project.name,
                    description: project.description.unwrap_or_default(),
                    organization_id: project.organization.key().to_string(),
                    is_active: project.is_active,
                    created_at: project.created_at.to_rfc3339(),
                    updated_at: project.updated_at.to_rfc3339(),
                })
                .collect(),
            page,
            page_size,
        }))
    }

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<ProjectResponse>, Status> {
        let user_id = extract_user_from_token(&request).await?;
        let UpdateProjectRequest {
            project_id,
            name,
            description,
            is_active,
        } = request.into_inner();
        let project_id_str = format!("projects:{}", project_id);

        // check write permission on project
        check_permission(
            &user_id,
            &project_id_str,
            permissions::resources::PROJECTS,
            permissions::actions::WRITE,
        )
        .await?;

        let proj_id = project_id.into();
        let patch = ProjectPatch {
            name,
            description,
            is_active,
        };
        let project = ProjectService::<ProjRepo, UserRepo, OrgRepo>::update_project(proj_id, patch)
            .await
            .map_err(map_err)?;
        Ok(Response::new(ProjectResponse {
            project_id: project.id.key().to_string(),
            name: project.name,
            description: project.description.unwrap_or_default(),
            organization_id: project.organization.key().to_string(),
            is_active: project.is_active,
            created_at: project.created_at.to_rfc3339(),
            updated_at: project.updated_at.to_rfc3339(),
        }))
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<DeleteProjectResponse>, Status> {
        let user_id = extract_user_from_token(&request).await?;
        let DeleteProjectRequest { project_id } = request.into_inner();
        let project_id_str = format!("projects:{}", project_id);

        // check delete permission on project
        check_permission(
            &user_id,
            &project_id_str,
            permissions::resources::PROJECTS,
            permissions::actions::DELETE,
        )
        .await?;

        let proj_id = project_id.into();
        ProjectService::<ProjRepo, UserRepo, OrgRepo>::deactivate_project(proj_id)
            .await
            .map_err(map_err)?;
        Ok(Response::new(DeleteProjectResponse::default()))
    }

    async fn add_user_to_project(
        &self,
        request: Request<AddUserToProjectRequest>,
    ) -> Result<Response<AddUserToProjectResponse>, Status> {
        let requester_id = extract_user_from_token(&request).await?;
        let AddUserToProjectRequest {
            user_id,
            project_id,
            role,
        } = request.into_inner();
        let project_id_str = format!("projects:{}", project_id);

        // check manage_users permission on project
        check_permission(
            &requester_id,
            &project_id_str,
            permissions::resources::PROJECTS,
            permissions::actions::MANAGE_USERS,
        )
        .await?;

        let user_record_id = user_id.into();
        let project_record_id = project_id.into();
        let role = role.unwrap_or_else(|| "member".to_string());
        let relation = ProjectService::<ProjRepo, UserRepo, OrgRepo>::add_user_to_project(
            user_record_id,
            project_record_id,
            role,
        )
        .await
        .map_err(map_err)?;
        Ok(Response::new(AddUserToProjectResponse {
            relation_id: relation.id.key().to_string(),
        }))
    }

    async fn remove_user_from_project(
        &self,
        request: Request<RemoveUserFromProjectRequest>,
    ) -> Result<Response<RemoveUserFromProjectResponse>, Status> {
        let requester_id = extract_user_from_token(&request).await?;
        let RemoveUserFromProjectRequest {
            user_id,
            project_id,
        } = request.into_inner();
        let project_id_str = format!("projects:{}", project_id);

        // check manage_users permission on project
        check_permission(
            &requester_id,
            &project_id_str,
            permissions::resources::PROJECTS,
            permissions::actions::MANAGE_USERS,
        )
        .await?;

        let user_record_id = user_id.into();
        let project_record_id = project_id.into();
        ProjectService::<ProjRepo, UserRepo, OrgRepo>::remove_user_from_project(
            user_record_id,
            project_record_id,
        )
        .await
        .map_err(map_err)?;
        Ok(Response::new(RemoveUserFromProjectResponse::default()))
    }

    async fn list_project_users(
        &self,
        request: Request<ListProjectUsersRequest>,
    ) -> Result<Response<ListProjectUsersResponse>, Status> {
        let user_id = extract_user_from_token(&request).await?;
        let ListProjectUsersRequest {
            project_id,
            page,
            page_size,
        } = request.into_inner();
        let project_id_str = format!("projects:{}", project_id);

        // check read permission on project to view members
        check_permission(
            &user_id,
            &project_id_str,
            permissions::resources::PROJECTS,
            permissions::actions::READ,
        )
        .await?;

        let proj_id = project_id.into();
        let page_u32 =
            u32::try_from(page).map_err(|_| Status::invalid_argument("page is too big"))?;
        let page_size_u32 = u32::try_from(page_size)
            .map_err(|_| Status::invalid_argument("page_size is too big"))?;
        let (users, total_count) =
            ProjectService::<ProjRepo, UserRepo, OrgRepo>::list_project_users(
                proj_id,
                page_u32,
                page_size_u32,
            )
            .await
            .map_err(map_err)?;
        Ok(Response::new(ListProjectUsersResponse {
            total_count: total_count as u64,
            users: users
                .into_iter()
                .map(|(user, relation)| ProjectUserInfo {
                    user_id: user.id.key().to_string(),
                    username: user.username.to_string(),
                    role: relation.role,
                    is_active: user.is_active,
                    joined_at: relation.joined_at.to_rfc3339(),
                })
                .collect(),
            page,
            page_size,
        }))
    }

    async fn list_user_projects(
        &self,
        request: Request<ListUserProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let ListUserProjectsRequest {
            user_id,
            page,
            page_size,
        } = request.into_inner();
        let user_record_id = user_id.into();
        let page_u32 =
            u32::try_from(page).map_err(|_| Status::invalid_argument("page is too big"))?;
        let page_size_u32 = u32::try_from(page_size)
            .map_err(|_| Status::invalid_argument("page_size is too big"))?;
        let (projects, total_count) =
            ProjectService::<ProjRepo, UserRepo, OrgRepo>::list_user_projects(
                user_record_id,
                page_u32,
                page_size_u32,
            )
            .await
            .map_err(map_err)?;
        Ok(Response::new(ListProjectsResponse {
            total_count: total_count as u64,
            projects: projects
                .into_iter()
                .map(|(project, _relation)| ProjectResponse {
                    project_id: project.id.key().to_string(),
                    name: project.name,
                    description: project.description.unwrap_or_default(),
                    organization_id: project.organization.key().to_string(),
                    is_active: project.is_active,
                    created_at: project.created_at.to_rfc3339(),
                    updated_at: project.updated_at.to_rfc3339(),
                })
                .collect(),
            page,
            page_size,
        }))
    }
}

fn map_err(e: ProjectDomainError) -> Status {
    use ProjectDomainError as E;
    match e {
        E::Validation(msg) => Status::invalid_argument(msg),
        E::InvalidPagination { field } => {
            Status::invalid_argument(format!("invalid pagination parameter: {}", field))
        }
        E::ProjectNotFound => Status::not_found("Project not found"),
        E::UserNotFound => Status::not_found("User not found"),
        E::OrganizationNotFound => Status::not_found("Organization not found"),
        E::UserNotInOrganization => Status::permission_denied(
            "User must be a member of the project's organization to be added to the project",
        ),
        E::Repo(e) => Status::internal(format!("Repository error: {}", e)),
    }
}
