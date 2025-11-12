use crate::application::dto::{
    AddUserToOrganizationRequestDto, CreateOrganizationRequestDto, DeleteOrganizationRequestDto,
    GetOrganizationRequestDto, ListOrganizationUsersRequestDto, ListOrganizationsRequestDto,
    ListUserOrganizationsRequestDto, RemoveUserFromOrganizationRequestDto,
    ToggleActiveOrganizationRequestDto, UpdateOrganizationRequestDto,
};
use crate::domain::value_objects::{
    Description, OrganizationId, OrganizationName, UserId, UserOrganizationRole,
};
use crate::presentation::grpc::mappers::{domain_to_proto_metadata, proto_to_domain_pagination};
use crate::presentation::grpc::middleware::check_permissions;
use crate::shared::di::AppContainer;
use derive_more::Constructor;
use protocol::services::organization::{
    AddUserToOrganizationRequest, AddUserToOrganizationResponse, CreateOrganizationRequest,
    DeleteOrganizationRequest, DeleteOrganizationResponse, GetOrganizationRequest,
    ListOrganizationUsersRequest, ListOrganizationUsersResponse, ListOrganizationsRequest,
    ListOrganizationsResponse, ListUserOrganizationsRequest, ListUserOrganizationsResponse,
    OrganizationResponse, RemoveUserFromOrganizationRequest, RemoveUserFromOrganizationResponse,
    ToggleOrganizationActiveRequest, ToggleOrganizationActiveResponse, UpdateOrganizationRequest,
    organization_service_server,
};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

#[derive(Constructor)]
pub struct OrganizationHandler {
    container: Arc<AppContainer>,
}

#[async_trait::async_trait]
impl organization_service_server::OrganizationService for OrganizationHandler {
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        // Check RBAC permissions (token already validated by interceptor)
        let auth_ctx = check_permissions(
            &request,
            self.container.rbac_enforcer(),
            "*",
            "organizations",
            "create",
        )
        .await?;

        let req = request.into_inner();
        let dto = CreateOrganizationRequestDto {
            name: OrganizationName::new(req.name)?,
            description: req.description.map(|s| Description::new(s)).transpose()?,
            creator_id: auth_ctx.user_id,
        };

        let response = self
            .container
            .create_organization_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn get_organization(
        &self,
        request: Request<GetOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        // let org_id = &request.get_ref().organization_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     org_id,
        //     "organizations",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = GetOrganizationRequestDto {
            organization_id: OrganizationId::new(req.organization_id),
        };

        let response = self
            .container
            .get_organization_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn update_organization(
        &self,
        request: Request<UpdateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        // let org_id = &request.get_ref().organization_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     org_id,
        //     "organizations",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();

        let dto = UpdateOrganizationRequestDto {
            organization_id: OrganizationId::new(req.organization_id),
            name: req.name.map(|s| OrganizationName::new(s)).transpose()?,
            description: req.description.map(|s| Description::new(s)).transpose()?,
        };

        let response = self
            .container
            .update_organization_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn toggle_organization_active(
        &self,
        request: Request<ToggleOrganizationActiveRequest>,
    ) -> Result<Response<ToggleOrganizationActiveResponse>, Status> {
        // let org_id = &request.get_ref().organization_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     org_id,
        //     "organizations",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = ToggleActiveOrganizationRequestDto {
            organization_id: OrganizationId::new(req.organization_id),
        };

        let _response = self
            .container
            .toggle_organization_active_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(ToggleOrganizationActiveResponse::default()))
    }

    async fn delete_organization(
        &self,
        request: Request<DeleteOrganizationRequest>,
    ) -> Result<Response<DeleteOrganizationResponse>, Status> {
        // let org_id = &request.get_ref().organization_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     org_id,
        //     "organizations",
        //     "delete",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = DeleteOrganizationRequestDto {
            organization_id: OrganizationId::new(req.organization_id),
        };

        let _response = self
            .container
            .delete_organization_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(DeleteOrganizationResponse::default()))
    }

    async fn list_organizations(
        &self,
        request: Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        // Check RBAC permissions for listing all organizations
        check_permissions(
            &request,
            self.container.rbac_enforcer(),
            "*",
            "organizations",
            "read",
        )
        .await?;

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let response = self
            .container
            .list_organizations_use_case()
            .execute(ListOrganizationsRequestDto { pagination })
            .await?;

        let organizations = response.organizations.into_iter().map(Into::into).collect();
        let pagination = response.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListOrganizationsResponse {
            organizations,
            pagination,
        }))
    }

    async fn list_organization_users(
        &self,
        request: Request<ListOrganizationUsersRequest>,
    ) -> Result<Response<ListOrganizationUsersResponse>, Status> {
        // let org_id = &request.get_ref().organization_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     org_id,
        //     "organizations",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let dto = ListOrganizationUsersRequestDto {
            organization_id: OrganizationId::new(req.organization_id),
            pagination,
        };

        let response = self
            .container
            .list_organization_users_use_case()
            .execute(dto)
            .await?;

        let users = response.users.into_iter().map(Into::into).collect();
        let pagination = response.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListOrganizationUsersResponse {
            users,
            pagination,
        }))
    }

    async fn list_user_organizations(
        &self,
        request: Request<ListUserOrganizationsRequest>,
    ) -> Result<Response<ListUserOrganizationsResponse>, Status> {
        // Check RBAC permissions for listing user organizations
        check_permissions(
            &request,
            self.container.rbac_enforcer(),
            "*",
            "organizations",
            "read",
        )
        .await?;

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let dto = ListUserOrganizationsRequestDto {
            user_id: UserId::new(req.user_id),
            pagination,
        };

        let response = self
            .container
            .list_user_organizations_use_case()
            .execute(dto)
            .await?;

        let organizations = response.organizations.into_iter().map(Into::into).collect();
        let pagination = response.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListUserOrganizationsResponse {
            organizations,
            pagination,
        }))
    }

    async fn add_user_to_organization(
        &self,
        request: Request<AddUserToOrganizationRequest>,
    ) -> Result<Response<AddUserToOrganizationResponse>, Status> {
        // let org_id = &request.get_ref().organization_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     org_id,
        //     "organizations",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = AddUserToOrganizationRequestDto {
            user_id: UserId::new(req.user_id),
            organization_id: OrganizationId::new(req.organization_id),
            role: UserOrganizationRole::new(req.role)?,
        };

        let response = self
            .container
            .add_user_to_organization_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(AddUserToOrganizationResponse {
            relation_id: response.relation_id.to_string(),
        }))
    }

    async fn remove_user_from_organization(
        &self,
        request: Request<RemoveUserFromOrganizationRequest>,
    ) -> Result<Response<RemoveUserFromOrganizationResponse>, Status> {
        // let org_id = &request.get_ref().organization_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     org_id,
        //     "organizations",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = RemoveUserFromOrganizationRequestDto {
            organization_id: OrganizationId::new(req.organization_id),
            user_id: UserId::new(req.user_id),
        };

        let _response = self
            .container
            .remove_user_from_organization_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(RemoveUserFromOrganizationResponse::default()))
    }
}
