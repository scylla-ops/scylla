use crate::extract_auth_context;
use crate::grpc::convert::{required, wrap};
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
use scylla_protocol::organization::v1::{
    AddOrganizationMemberRequest, AddOrganizationMemberResponse, CreateOrganizationRequest,
    CreateOrganizationResponse, DeleteOrganizationRequest, DeleteOrganizationResponse,
    GetOrganizationRequest, GetOrganizationResponse, ListOrganizationMembersRequest,
    ListOrganizationMembersResponse, ListOrganizationsRequest, ListOrganizationsResponse,
    ListUserOrganizationsRequest, ListUserOrganizationsResponse, Organization as ProtoOrganization,
    OrganizationMember, RemoveOrganizationMemberRequest, RemoveOrganizationMemberResponse,
    SetOrganizationActiveRequest, SetOrganizationActiveResponse, UpdateOrganizationRequest,
    UpdateOrganizationResponse, organization_service_server::OrganizationService,
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
    ) -> Result<Response<CreateOrganizationResponse>, Status> {
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

        Ok(Response::new(CreateOrganizationResponse {
            organization: Some(organization_to_proto(&org)),
        }))
    }

    async fn get_organization(
        &self,
        request: Request<GetOrganizationRequest>,
    ) -> Result<Response<GetOrganizationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&required(req.organization_id, "organization_id")?);

        let org = self
            .use_cases
            .get(&caller, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(GetOrganizationResponse {
            organization: Some(organization_to_proto(&org)),
        }))
    }

    async fn update_organization(
        &self,
        request: Request<UpdateOrganizationRequest>,
    ) -> Result<Response<UpdateOrganizationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&required(req.organization_id, "organization_id")?);

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

        Ok(Response::new(UpdateOrganizationResponse {
            organization: Some(organization_to_proto(&org)),
        }))
    }

    /// Sets the active flag to the requested value, so a retried call lands on
    /// the state the caller asked for instead of flipping it back.
    async fn set_organization_active(
        &self,
        request: Request<SetOrganizationActiveRequest>,
    ) -> Result<Response<SetOrganizationActiveResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&required(req.organization_id, "organization_id")?);

        let org = self
            .use_cases
            .set_active(&caller, &org_id, req.is_active)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(SetOrganizationActiveResponse {
            organization: Some(organization_to_proto(&org)),
        }))
    }

    async fn delete_organization(
        &self,
        request: Request<DeleteOrganizationRequest>,
    ) -> Result<Response<DeleteOrganizationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&required(req.organization_id, "organization_id")?);

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
        let organizations: Vec<ProtoOrganization> =
            orgs.iter().map(organization_to_proto).collect();

        Ok(Response::new(ListOrganizationsResponse {
            organizations,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_organization_members(
        &self,
        request: Request<ListOrganizationMembersRequest>,
    ) -> Result<Response<ListOrganizationMembersResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&required(req.organization_id, "organization_id")?);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (users, metadata) = self
            .use_cases
            .list_users(&caller, &org_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let members = users
            .iter()
            .map(|user| OrganizationMember {
                user_id: wrap(user.id().to_string()),
                username: user.username().to_string(),
            })
            .collect();

        Ok(Response::new(ListOrganizationMembersResponse {
            members,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_user_organizations(
        &self,
        request: Request<ListUserOrganizationsRequest>,
    ) -> Result<Response<ListUserOrganizationsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let user_id = UserId::new(&required(req.user_id, "user_id")?);
        let pagination = proto_to_domain_pagination(req.pagination);

        let (orgs, metadata) = self
            .use_cases
            .list_user_orgs(&caller, &user_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let organizations: Vec<ProtoOrganization> =
            orgs.iter().map(organization_to_proto).collect();

        Ok(Response::new(ListUserOrganizationsResponse {
            organizations,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn add_organization_member(
        &self,
        request: Request<AddOrganizationMemberRequest>,
    ) -> Result<Response<AddOrganizationMemberResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&required(req.organization_id, "organization_id")?);
        let user_id = UserId::new(&required(req.user_id, "user_id")?);

        self.use_cases
            .add_user(&caller, &user_id, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(AddOrganizationMemberResponse {}))
    }

    async fn remove_organization_member(
        &self,
        request: Request<RemoveOrganizationMemberRequest>,
    ) -> Result<Response<RemoveOrganizationMemberResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(&required(req.organization_id, "organization_id")?);
        let user_id = UserId::new(&required(req.user_id, "user_id")?);

        self.use_cases
            .remove_user(&caller, &user_id, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RemoveOrganizationMemberResponse {}))
    }
}
