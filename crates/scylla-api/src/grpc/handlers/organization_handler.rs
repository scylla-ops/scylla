use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, organization_to_proto,
    proto_to_domain_pagination,
};
use derive_more::Constructor;
use scylla_core::application::OrganizationUseCases;
use scylla_core::application::authz::policy::PolicyControl;
use scylla_core::application::{
    OrganizationRepository, PermissionService, UserOrganizationRepository, UserRepository,
};
use scylla_core::domain::entities::{OrganizationId, UserId};
use scylla_core::domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use scylla_protocol::services::organization::{
    AddUserToOrganizationRequest, AddUserToOrganizationResponse, CreateOrganizationRequest,
    DeleteOrganizationRequest, DeleteOrganizationResponse, GetOrganizationRequest,
    ListOrganizationUsersRequest, ListOrganizationUsersResponse, ListOrganizationsRequest,
    ListOrganizationsResponse, ListUserOrganizationsRequest, ListUserOrganizationsResponse,
    OrganizationResponse, OrganizationUserInfoResponse, RemoveUserFromOrganizationRequest,
    RemoveUserFromOrganizationResponse, ToggleOrganizationActiveRequest,
    ToggleOrganizationActiveResponse, UpdateOrganizationRequest,
    organization_service_server::OrganizationService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct OrganizationHandler<
    O: OrganizationRepository,
    UO: UserOrganizationRepository,
    U: UserRepository,
    PS: PermissionService,
    PC: PolicyControl,
> {
    use_cases: Arc<OrganizationUseCases<O, UO, U, PS, PC>>,
}

#[async_trait::async_trait]
impl<
    O: OrganizationRepository + Send + Sync + 'static,
    UO: UserOrganizationRepository + Send + Sync + 'static,
    U: UserRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
> OrganizationService for OrganizationHandler<O, UO, U, PS, PC>
{
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let name = OrganizationName::new(&req.name).map_err(domain_error_to_status)?;
        let description = req
            .description
            .map(|d| OrganizationDescription::new(&d))
            .transpose()
            .map_err(domain_error_to_status)?;

        let org = self
            .use_cases
            .create(&caller, name, description)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn get_organization(
        &self,
        request: Request<GetOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&req.organization_id);

        let org = self
            .use_cases
            .get(&caller, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn update_organization(
        &self,
        request: Request<UpdateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&req.organization_id);

        let name = req
            .name
            .map(|n| OrganizationName::new(&n))
            .transpose()
            .map_err(domain_error_to_status)?;
        let description = req
            .description
            .map(|d| OrganizationDescription::new(&d).map(Some))
            .transpose()
            .map_err(domain_error_to_status)?;

        let org = self
            .use_cases
            .update(&caller, &org_id, name, description)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn toggle_organization_active(
        &self,
        request: Request<ToggleOrganizationActiveRequest>,
    ) -> Result<Response<ToggleOrganizationActiveResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&req.organization_id);

        self.use_cases
            .toggle_active(&caller, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ToggleOrganizationActiveResponse {}))
    }

    async fn delete_organization(
        &self,
        request: Request<DeleteOrganizationRequest>,
    ) -> Result<Response<DeleteOrganizationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&req.organization_id);

        self.use_cases
            .delete(&caller, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteOrganizationResponse {}))
    }

    async fn list_organizations(
        &self,
        request: Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list(&caller, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (orgs, metadata) = result.into_parts();
        let organizations: Vec<OrganizationResponse> =
            orgs.iter().map(organization_to_proto).collect();

        Ok(Response::new(ListOrganizationsResponse {
            organizations,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_organization_users(
        &self,
        request: Request<ListOrganizationUsersRequest>,
    ) -> Result<Response<ListOrganizationUsersResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&req.organization_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (users, metadata) = self
            .use_cases
            .list_users(&caller, &org_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let users = users
            .iter()
            .map(|user| OrganizationUserInfoResponse {
                user_id: user.id().to_string(),
                username: user.username().to_string(),
            })
            .collect();

        Ok(Response::new(ListOrganizationUsersResponse {
            users,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_user_organizations(
        &self,
        request: Request<ListUserOrganizationsRequest>,
    ) -> Result<Response<ListUserOrganizationsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (orgs, metadata) = self
            .use_cases
            .list_user_orgs(&caller, &user_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let organizations: Vec<OrganizationResponse> =
            orgs.iter().map(organization_to_proto).collect();

        Ok(Response::new(ListUserOrganizationsResponse {
            organizations,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn add_user_to_organization(
        &self,
        request: Request<AddUserToOrganizationRequest>,
    ) -> Result<Response<AddUserToOrganizationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&req.organization_id);
        let user_id = UserId::new(&req.user_id);

        self.use_cases
            .add_user(&caller, &user_id, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(AddUserToOrganizationResponse {}))
    }

    async fn remove_user_from_organization(
        &self,
        request: Request<RemoveUserFromOrganizationRequest>,
    ) -> Result<Response<RemoveUserFromOrganizationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&req.organization_id);
        let user_id = UserId::new(&req.user_id);

        self.use_cases
            .remove_user(&caller, &user_id, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RemoveUserFromOrganizationResponse {}))
    }
}
