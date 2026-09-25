//! The adapter: each RPC is one `run` and its response. Parsing lives in the
//! user mapper, behind `Parse`; no RPC checks a permission or touches a port.

use crate::application::UserUseCases;
use crate::grpc::adapter::run;
use crate::grpc::mappers::user_to_proto;
use derive_more::Constructor;
use scylla_extension::Actions;
use scylla_proto::user::v1::{
    CreateUserRequest, CreateUserResponse, DeleteUserRequest, DeleteUserResponse, GetUserRequest,
    GetUserResponse, ListUsersRequest, ListUsersResponse, UpdateUserRequest, UpdateUserResponse,
    user_service_server::UserService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct UserHandler {
    actions: Arc<Actions>,
    users: Arc<UserUseCases>,
}

#[async_trait::async_trait]
impl UserService for UserHandler {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let user = run(&self.actions, &*self.users, request).await?;
        Ok(Response::new(CreateUserResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let user = run(&self.actions, &*self.users, request).await?;
        Ok(Response::new(GetUserResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let user = run(&self.actions, &*self.users, request).await?;
        Ok(Response::new(UpdateUserResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        run(&self.actions, &*self.users, request).await?;
        Ok(Response::new(DeleteUserResponse {}))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let page = run(&self.actions, &*self.users, request).await?;
        Ok(Response::new(page.into()))
    }
}
