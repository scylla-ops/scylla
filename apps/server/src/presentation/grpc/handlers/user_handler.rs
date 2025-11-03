use crate::application::dto::{
    ChangeUserGlobalRoleRequestDto, CreateUserRequestDto, DeleteUserRequestDto, GetUserRequestDto,
    ListUsersRequestDto, UpdateUserRequestDto,
};
use crate::domain::value_objects::{Password, UserGlobalRole, UserId, Username};
use crate::presentation::grpc::mappers::{
    domain_to_proto_metadata, map_domain_error_to_status, proto_to_domain_pagination,
};
use crate::presentation::grpc::middleware::check_permissions;
use crate::shared::di::AppContainer;
use derive_more::Constructor;
use protocol::services::user::{
    ChangeUserGlobalRoleRequest, ChangeUserGlobalRoleResponse, CreateUserRequest,
    DeleteUserRequest, DeleteUserResponse, GetUserRequest, ListUsersRequest, ListUsersResponse,
    UpdateUserRequest, UserResponse, user_service_server,
};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

#[derive(Constructor)]
pub struct UserHandler {
    container: Arc<AppContainer>,
}

#[async_trait::async_trait]
impl user_service_server::UserService for UserHandler {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        // Check RBAC permissions for creating users (admin only)
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     "*",
        //     "users",
        //     "create",
        // )
        // .await?;

        let req = request.into_inner();

        let dto = CreateUserRequestDto {
            username: Username::new(req.username).map_err(map_domain_error_to_status)?,
            password: Password::new(req.password).map_err(map_domain_error_to_status)?,
        };

        let response = self
            .container
            .create_user_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(response.into()))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        // let user_id = &request.get_ref().user_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     user_id,
        //     "users",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = GetUserRequestDto {
            user_id: UserId::new(req.user_id),
        };

        let response = self
            .container
            .get_user_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(response.into()))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        // let user_id = &request.get_ref().user_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     user_id,
        //     "users",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();

        // Convert optional string fields to optional value objects
        let dto = UpdateUserRequestDto {
            user_id: UserId::new(req.user_id),
            username: req
                .username
                .filter(|s| !s.is_empty())
                .map(Username::new)
                .transpose()
                .map_err(map_domain_error_to_status)?,
        };

        let response = self
            .container
            .update_user_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(response.into()))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        // let user_id = &request.get_ref().user_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     user_id,
        //     "users",
        //     "delete",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = DeleteUserRequestDto {
            user_id: UserId::new(req.user_id),
        };

        let _response = self
            .container
            .delete_user_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(DeleteUserResponse::default()))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        // Check RBAC permissions for listing all users
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     "*",
        //     "users",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let response = self
            .container
            .list_users_use_case()
            .execute(ListUsersRequestDto { pagination })
            .await
            .map_err(map_domain_error_to_status)?;

        let users: Vec<UserResponse> = response.users.into_iter().map(Into::into).collect();
        let pagination = response.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListUsersResponse { users, pagination }))
    }

    async fn change_user_global_role(
        &self,
        request: Request<ChangeUserGlobalRoleRequest>,
    ) -> Result<Response<ChangeUserGlobalRoleResponse>, Status> {
        // Check RBAC permissions for changing global roles (admin only)
        let auth_ctx = check_permissions(
            &request,
            self.container.rbac_enforcer(),
            "*",
            "users",
            "update",
        )
        .await?;

        let req = request.into_inner();
        let new_role = UserGlobalRole::new(req.new_role).map_err(map_domain_error_to_status)?;

        let dto = ChangeUserGlobalRoleRequestDto {
            user_id: UserId::new(req.user_id),
            new_role,
            caller_id: Some(auth_ctx.user_id),
        };

        let response = self
            .container
            .change_user_global_role_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(ChangeUserGlobalRoleResponse {
            user_id: response.user_id.to_string(),
            new_role: response.new_role.to_string(),
        }))
    }
}
