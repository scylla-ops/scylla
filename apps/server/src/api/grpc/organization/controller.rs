use crate::api::grpc::organization::models::OrganizationPatch;
#[cfg(feature = "surreal")]
use crate::api::grpc::organization::repos::surreal::OrganizationRepositorySurreal;
use crate::api::grpc::organization::service::{OrganizationDomainError, OrganizationService};
#[cfg(feature = "surreal")]
use crate::api::grpc::user::repos::surreal::UserRepositorySurreal;
#[cfg(feature = "surreal")]
use protocol::services::organization::{
    AddUserToOrganizationRequest, AddUserToOrganizationResponse, CreateOrganizationRequest,
    DeleteOrganizationRequest, DeleteOrganizationResponse, GetOrganizationRequest,
    ListOrganizationUsersRequest, ListOrganizationUsersResponse, ListOrganizationsRequest,
    ListOrganizationsResponse, ListUserOrganizationsRequest, ListUserOrganizationsResponse,
    OrganizationResponse, RemoveUserFromOrganizationRequest, RemoveUserFromOrganizationResponse,
    UpdateOrganizationRequest, UserInfo, organization_service_server,
};
use protocol::tonic::{Request, Response, Status};

#[cfg(feature = "surreal")]
type OrgRepo = OrganizationRepositorySurreal;

#[cfg(feature = "surreal")]
type UserRepo = UserRepositorySurreal;

pub struct OrganizationController;

#[async_trait::async_trait]
impl organization_service_server::OrganizationService for OrganizationController {
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let CreateOrganizationRequest { name, description } = request.into_inner();
        let org = OrganizationService::<OrgRepo, UserRepo>::create_organization(name, description)
            .await
            .map_err(map_err)?;
        Ok(Response::new(OrganizationResponse {
            organization_id: org.id.key().to_string(),
            name: org.name,
            description: org.description.unwrap_or_default(),
            is_active: org.is_active,
            created_at: org.created_at.to_rfc3339(),
            updated_at: org.updated_at.to_rfc3339(),
        }))
    }

    async fn get_organization(
        &self,
        request: Request<GetOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let GetOrganizationRequest { organization_id } = request.into_inner();
        let org_id = organization_id.into();
        let org = OrganizationService::<OrgRepo, UserRepo>::get_organization(org_id)
            .await
            .map_err(map_err)?;
        Ok(Response::new(OrganizationResponse {
            organization_id: org.id.key().to_string(),
            name: org.name,
            description: org.description.unwrap_or_default(),
            is_active: org.is_active,
            created_at: org.created_at.to_rfc3339(),
            updated_at: org.updated_at.to_rfc3339(),
        }))
    }

    async fn list_organizations(
        &self,
        request: Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        let ListOrganizationsRequest { page, page_size } = request.into_inner();
        let page_u32 =
            u32::try_from(page).map_err(|_| Status::invalid_argument("page is too big"))?;
        let page_size_u32 = u32::try_from(page_size)
            .map_err(|_| Status::invalid_argument("page_size is too big"))?;
        let (orgs, total_count) =
            OrganizationService::<OrgRepo, UserRepo>::list_organizations(page_u32, page_size_u32)
                .await
                .map_err(map_err)?;
        Ok(Response::new(ListOrganizationsResponse {
            total_count: total_count as u64,
            organizations: orgs
                .into_iter()
                .map(|org| OrganizationResponse {
                    organization_id: org.id.key().to_string(),
                    name: org.name,
                    description: org.description.unwrap_or_default(),
                    is_active: org.is_active,
                    created_at: org.created_at.to_rfc3339(),
                    updated_at: org.updated_at.to_rfc3339(),
                })
                .collect(),
            page,
            page_size,
        }))
    }

    async fn update_organization(
        &self,
        request: Request<UpdateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let UpdateOrganizationRequest {
            organization_id,
            name,
            description,
            is_active,
        } = request.into_inner();
        let org_id = organization_id.into();
        let patch = OrganizationPatch {
            name,
            description,
            is_active,
        };
        let org = OrganizationService::<OrgRepo, UserRepo>::update_organization(org_id, patch)
            .await
            .map_err(map_err)?;
        Ok(Response::new(OrganizationResponse {
            organization_id: org.id.key().to_string(),
            name: org.name,
            description: org.description.unwrap_or_default(),
            is_active: org.is_active,
            created_at: org.created_at.to_rfc3339(),
            updated_at: org.updated_at.to_rfc3339(),
        }))
    }

    async fn delete_organization(
        &self,
        request: Request<DeleteOrganizationRequest>,
    ) -> Result<Response<DeleteOrganizationResponse>, Status> {
        let DeleteOrganizationRequest { organization_id } = request.into_inner();
        let org_id = organization_id.into();
        OrganizationService::<OrgRepo, UserRepo>::deactivate_organization(org_id)
            .await
            .map_err(map_err)?;
        Ok(Response::new(DeleteOrganizationResponse::default()))
    }

    async fn add_user_to_organization(
        &self,
        request: Request<AddUserToOrganizationRequest>,
    ) -> Result<Response<AddUserToOrganizationResponse>, Status> {
        let AddUserToOrganizationRequest {
            user_id,
            organization_id,
            role,
        } = request.into_inner();
        let user_record_id = user_id.into();
        let org_record_id = organization_id.into();
        let role = role.unwrap_or_else(|| "member".to_string());
        let relation = OrganizationService::<OrgRepo, UserRepo>::add_user_to_organization(
            user_record_id,
            org_record_id,
            role,
        )
        .await
        .map_err(map_err)?;
        Ok(Response::new(AddUserToOrganizationResponse {
            relation_id: relation.id.key().to_string(),
        }))
    }

    async fn remove_user_from_organization(
        &self,
        request: Request<RemoveUserFromOrganizationRequest>,
    ) -> Result<Response<RemoveUserFromOrganizationResponse>, Status> {
        let RemoveUserFromOrganizationRequest {
            user_id,
            organization_id,
        } = request.into_inner();
        let user_record_id = user_id.into();
        let org_record_id = organization_id.into();
        OrganizationService::<OrgRepo, UserRepo>::remove_user_from_organization(
            user_record_id,
            org_record_id,
        )
        .await
        .map_err(map_err)?;
        Ok(Response::new(RemoveUserFromOrganizationResponse::default()))
    }

    async fn list_organization_users(
        &self,
        request: Request<ListOrganizationUsersRequest>,
    ) -> Result<Response<ListOrganizationUsersResponse>, Status> {
        let ListOrganizationUsersRequest {
            organization_id,
            page,
            page_size,
        } = request.into_inner();
        let org_id = organization_id.into();
        let page_u32 =
            u32::try_from(page).map_err(|_| Status::invalid_argument("page is too big"))?;
        let page_size_u32 = u32::try_from(page_size)
            .map_err(|_| Status::invalid_argument("page_size is too big"))?;
        let (users, total_count) =
            OrganizationService::<OrgRepo, UserRepo>::list_organization_users(
                org_id,
                page_u32,
                page_size_u32,
            )
            .await
            .map_err(map_err)?;
        Ok(Response::new(ListOrganizationUsersResponse {
            total_count: total_count as u64,
            users: users
                .into_iter()
                .map(|(user, relation)| UserInfo {
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

    async fn list_user_organizations(
        &self,
        request: Request<ListUserOrganizationsRequest>,
    ) -> Result<Response<ListUserOrganizationsResponse>, Status> {
        let ListUserOrganizationsRequest {
            user_id,
            page,
            page_size,
        } = request.into_inner();
        let user_record_id = user_id.into();
        let page_u32 =
            u32::try_from(page).map_err(|_| Status::invalid_argument("page is too big"))?;
        let page_size_u32 = u32::try_from(page_size)
            .map_err(|_| Status::invalid_argument("page_size is too big"))?;
        let (orgs, total_count) =
            OrganizationService::<OrgRepo, UserRepo>::list_user_organizations(
                user_record_id,
                page_u32,
                page_size_u32,
            )
            .await
            .map_err(map_err)?;
        Ok(Response::new(ListUserOrganizationsResponse {
            total_count: total_count as u64,
            organizations: orgs
                .into_iter()
                .map(|(org, _relation)| OrganizationResponse {
                    organization_id: org.id.key().to_string(),
                    name: org.name,
                    description: org.description.unwrap_or_default(),
                    is_active: org.is_active,
                    created_at: org.created_at.to_rfc3339(),
                    updated_at: org.updated_at.to_rfc3339(),
                })
                .collect(),
            page,
            page_size,
        }))
    }
}

fn map_err(e: OrganizationDomainError) -> Status {
    use OrganizationDomainError as E;
    match e {
        E::Validation(msg) => Status::invalid_argument(msg),
        E::InvalidPagination { field } => {
            Status::invalid_argument(format!("invalid pagination parameter: {}", field))
        }
        E::OrganizationNotFound => Status::not_found("Organization not found"),
        E::UserNotFound => Status::not_found("User not found"),
        E::OrganizationNameExists => Status::already_exists("Organization name already exists"),
        E::Repo(e) => Status::internal(format!("Repository error: {}", e)),
    }
}
