use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, organization_to_proto,
    proto_to_domain_pagination,
};
use application::OrganizationUseCases;
use derive_more::Constructor;
use domain::entities::{OrganizationId, UserId};
use domain::ports::services::permission_service::PermissionService;
use domain::ports::{OrganizationRepository, UserOrganizationRepository, UserRepository};
use domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use domain::value_objects::permission::policy;
use protocol::services::organization::{
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
> {
    use_cases: Arc<OrganizationUseCases<O, UO, U>>,
    permission_checker: Arc<PS>,
}

#[async_trait::async_trait]
impl<
    O: OrganizationRepository + Send + Sync + 'static,
    UO: UserOrganizationRepository + Send + Sync + 'static,
    U: UserRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> OrganizationService for OrganizationHandler<O, UO, U, PS>
{
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        require_permission!(self, request, policy::organization::create());
        let req = request.into_inner();

        let name = OrganizationName::new(&req.name).map_err(domain_error_to_status)?;
        let description = req
            .description
            .map(|d| OrganizationDescription::new(&d))
            .transpose()
            .map_err(domain_error_to_status)?;

        let org = self
            .use_cases
            .create(name, description)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn get_organization(
        &self,
        request: Request<GetOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(self, request, policy::organization::get(org_id.clone()));
        let _ = request.into_inner();

        let org = self
            .use_cases
            .get(&org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn update_organization(
        &self,
        request: Request<UpdateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(self, request, policy::organization::update(org_id.clone()));
        let req = request.into_inner();

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
            .update(&org_id, name, description)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn toggle_organization_active(
        &self,
        request: Request<ToggleOrganizationActiveRequest>,
    ) -> Result<Response<ToggleOrganizationActiveResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(
            self,
            request,
            policy::organization::toggle_active(org_id.clone())
        );
        let _ = request.into_inner();

        self.use_cases
            .toggle_active(&org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ToggleOrganizationActiveResponse {}))
    }

    async fn delete_organization(
        &self,
        request: Request<DeleteOrganizationRequest>,
    ) -> Result<Response<DeleteOrganizationResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(self, request, policy::organization::delete(org_id.clone()));
        let _ = request.into_inner();

        self.use_cases
            .delete(&org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteOrganizationResponse {}))
    }

    async fn list_organizations(
        &self,
        request: Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        require_permission!(self, request, policy::organization::list());
        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list(pagination.as_ref())
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
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(
            self,
            request,
            policy::organization::list_users(org_id.clone())
        );
        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let (users, metadata) = self
            .use_cases
            .list_users(&org_id, pagination.as_ref())
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
        let user_id = UserId::new(&request.get_ref().user_id);
        require_permission!(
            self,
            request,
            policy::organization::list_user_orgs(user_id.clone())
        );
        let pagination = proto_to_domain_pagination(request.get_ref().pagination.clone());

        let (orgs, metadata) = self
            .use_cases
            .list_user_orgs(&user_id, pagination.as_ref())
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
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(
            self,
            request,
            policy::organization::add_user_to_organization(org_id.clone())
        );
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);

        self.use_cases
            .add_user(&user_id, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(AddUserToOrganizationResponse {}))
    }

    async fn remove_user_from_organization(
        &self,
        request: Request<RemoveUserFromOrganizationRequest>,
    ) -> Result<Response<RemoveUserFromOrganizationResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(
            self,
            request,
            policy::organization::remove_user_from_organization(org_id.clone())
        );
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);

        self.use_cases
            .remove_user(&user_id, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RemoveUserFromOrganizationResponse {}))
    }
}
