use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, proto_to_domain_pagination, user_to_proto,
};
use protocol::services::user::{
    ChangeUserGlobalRoleRequest, ChangeUserGlobalRoleResponse, CreateUserRequest, DeleteUserRequest,
    DeleteUserResponse, GetUserRequest, ListUsersRequest, ListUsersResponse, UpdateUserRequest,
    UserResponse, user_service_server::UserService,
};
use application::UserUseCases;
use derive_more::Constructor;
use domain::entities::UserId;
use domain::ports::{HashService, UserRepository};
use domain::value_objects::user::{Password, UserName};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct UserHandler<U: UserRepository, H: HashService> {
    use_cases: Arc<UserUseCases<U, H>>,
}

#[async_trait::async_trait]
impl<U: UserRepository + 'static, H: HashService + 'static> UserService for UserHandler<U, H> {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();

        let username = UserName::new(&req.username).map_err(domain_error_to_status)?;
        let password = Password::new(&req.password).map_err(domain_error_to_status)?;

        let user = self
            .use_cases
            .create(username, password)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(user_to_proto(&user)))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);

        let user = self
            .use_cases
            .get(&user_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(user_to_proto(&user)))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);

        let username = req
            .username
            .map(|u| UserName::new(&u))
            .transpose()
            .map_err(domain_error_to_status)?;

        let user = self
            .use_cases
            .update(&user_id, username)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(user_to_proto(&user)))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);

        self.use_cases
            .delete(&user_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteUserResponse {}))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list(pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (users, metadata) = result.into_parts();
        let user_responses: Vec<UserResponse> = users.iter().map(user_to_proto).collect();

        Ok(Response::new(ListUsersResponse {
            users: user_responses,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn change_user_global_role(
        &self,
        _request: Request<ChangeUserGlobalRoleRequest>,
    ) -> Result<Response<ChangeUserGlobalRoleResponse>, Status>
    {
        Err(Status::unimplemented(
            "Global role management requires RBAC configuration",
        ))
    }
}
