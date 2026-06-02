use crate::extract_auth_context;
use crate::grpc::convert::{optional, required};
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, proto_to_domain_pagination, user_to_proto,
};
use derive_more::Constructor;
use scylla_core::application::UserUseCases;
use scylla_core::application::{HashService, PermissionService, UserRepository};
use scylla_core::domain::entities::UserId;
use scylla_core::domain::value_objects::user::{Email, Password, Username};
use scylla_protocol::services::user::{
    ChangeUserGlobalRoleRequest, ChangeUserGlobalRoleResponse, CreateUserRequest,
    DeleteUserRequest, DeleteUserResponse, GetUserRequest, ListUsersRequest, ListUsersResponse,
    UpdateUserRequest, UserResponse, user_service_server::UserService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct UserHandler<U: UserRepository, H: HashService, PS: PermissionService> {
    use_cases: Arc<UserUseCases<U, H, PS>>,
}

#[async_trait::async_trait]
impl<
    U: UserRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> UserService for UserHandler<U, H, PS>
{
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let username = Username::new(&req.username).map_err(domain_error_to_status)?;
        let password = Password::new(&req.password).map_err(domain_error_to_status)?;
        let email = optional(req.email)
            .as_deref()
            .map(Email::new)
            .transpose()
            .map_err(domain_error_to_status)?;

        let user = self
            .use_cases
            .create(&caller, username, email, password)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = UserId::new(&required(req.user_id, "user_id")?);
        let user = self
            .use_cases
            .get(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = UserId::new(&required(req.user_id, "user_id")?);
        let username = req
            .username
            .map(|u| Username::new(&u))
            .transpose()
            .map_err(domain_error_to_status)?;
        let user = self
            .use_cases
            .update(&caller, &id, username)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = UserId::new(&required(req.user_id, "user_id")?);
        self.use_cases
            .delete(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(DeleteUserResponse {}))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);
        let result = self
            .use_cases
            .list(&caller, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;
        let (users, metadata) = result.into_parts();

        Ok(Response::new(ListUsersResponse {
            users: users.iter().map(user_to_proto).collect(),
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn change_user_global_role(
        &self,
        _request: Request<ChangeUserGlobalRoleRequest>,
    ) -> Result<Response<ChangeUserGlobalRoleResponse>, Status> {
        Err(Status::unimplemented(
            "Global role management requires RBAC configuration",
        ))
    }
}
