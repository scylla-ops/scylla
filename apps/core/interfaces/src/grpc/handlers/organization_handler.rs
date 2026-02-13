use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, organization_to_proto,
    proto_to_domain_pagination,
};
use crate::grpc::services::services::organization::{
    AddUserToOrganizationRequest, AddUserToOrganizationResponse, CreateOrganizationRequest,
    DeleteOrganizationRequest, DeleteOrganizationResponse, GetOrganizationRequest,
    ListOrganizationUsersRequest, ListOrganizationUsersResponse, ListOrganizationsRequest,
    ListOrganizationsResponse, ListUserOrganizationsRequest, ListUserOrganizationsResponse,
    OrganizationResponse, OrganizationUserInfoResponse, RemoveUserFromOrganizationRequest,
    RemoveUserFromOrganizationResponse, ToggleOrganizationActiveRequest,
    ToggleOrganizationActiveResponse, UpdateOrganizationRequest,
    organization_service_server::OrganizationService,
};
use application::OrganizationUseCases;
use derive_more::Constructor;
use domain::entities::{OrganizationId, UserId};
use domain::ports::{OrganizationRepository, UserOrganizationRepository, UserRepository};
use domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct OrganizationHandler<
    O: OrganizationRepository,
    UO: UserOrganizationRepository,
    U: UserRepository,
> {
    use_cases: Arc<OrganizationUseCases<O, UO, U>>,
}

#[async_trait::async_trait]
impl<
    O: OrganizationRepository + 'static,
    UO: UserOrganizationRepository + 'static,
    U: UserRepository + 'static,
> OrganizationService for OrganizationHandler<O, UO, U>
{
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
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
        let req = request.into_inner();
        let id = OrganizationId::new(&req.organization_id);

        let org = self
            .use_cases
            .get(&id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn update_organization(
        &self,
        request: Request<UpdateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let req = request.into_inner();
        let id = OrganizationId::new(&req.organization_id);

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
            .update(&id, name, description)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn toggle_organization_active(
        &self,
        request: Request<ToggleOrganizationActiveRequest>,
    ) -> Result<Response<ToggleOrganizationActiveResponse>, Status> {
        let req = request.into_inner();
        let id = OrganizationId::new(&req.organization_id);

        self.use_cases
            .toggle_active(&id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ToggleOrganizationActiveResponse {}))
    }

    async fn delete_organization(
        &self,
        request: Request<DeleteOrganizationRequest>,
    ) -> Result<Response<DeleteOrganizationResponse>, Status> {
        let req = request.into_inner();
        let id = OrganizationId::new(&req.organization_id);

        self.use_cases
            .delete(&id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteOrganizationResponse {}))
    }

    async fn list_organizations(
        &self,
        request: Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
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
        let req = request.into_inner();
        let org_id = OrganizationId::new(&req.organization_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (pairs, metadata) = self
            .use_cases
            .list_users(&org_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let users = pairs
            .iter()
            .map(|(user, membership)| OrganizationUserInfoResponse {
                user_id: user.id().to_string(),
                username: user.username().to_string(),
                role: membership.role().as_str().to_string(),
                joined_at: membership.joined_at().to_rfc3339(),
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
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let pagination = proto_to_domain_pagination(req.pagination);

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
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let org_id = OrganizationId::new(&req.organization_id);

        let relation_id = self
            .use_cases
            .add_user(&user_id, &org_id, &req.role)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(AddUserToOrganizationResponse {
            relation_id: relation_id.to_string(),
        }))
    }

    async fn remove_user_from_organization(
        &self,
        request: Request<RemoveUserFromOrganizationRequest>,
    ) -> Result<Response<RemoveUserFromOrganizationResponse>, Status> {
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let org_id = OrganizationId::new(&req.organization_id);

        self.use_cases
            .remove_user(&user_id, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RemoveUserFromOrganizationResponse {}))
    }
}
