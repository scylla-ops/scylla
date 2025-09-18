use crate::api::grpc::user::service::UserService;
use crate::api::grpc::user::{
    dto::{NewUserRequest as DomainNewUserRequest, UpdateUserRequest as DomainUpdateUserRequest},
    service::UserDomainError,
};
use crate::parse_uuid;
use derive_more::Constructor;
use protocol::{
    services::{
        CreateUserRequest, DeleteUserRequest, DeleteUserResponse, GetUserRequest, ListUsersRequest,
        ListUsersResponse, UpdateUserRequest, UserResponse, user_service_server,
    },
    tonic::{Request, Response, Status},
};
use std::sync::Arc;

#[derive(Constructor)]
pub struct UserController {
    service: Arc<UserService>,
}

#[async_trait::async_trait]
impl user_service_server::UserService for UserController {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();
        let domain_req: DomainNewUserRequest = req.into();
        let user = self
            .service
            .create_user(domain_req)
            .await
            .map_err(map_err)?;
        Ok(Response::new(user.into()))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let GetUserRequest { user_uuid } = request.into_inner();
        let id = parse_uuid!(user_uuid)?;
        let user = self.service.get_user(id).await.map_err(map_err)?;
        Ok(Response::new(user.into()))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let ListUsersRequest { page, page_size } = request.into_inner();
        let page_u32 =
            u32::try_from(page).map_err(|_| Status::invalid_argument("page is too big"))?;
        let page_size_u32 = u32::try_from(page_size)
            .map_err(|_| Status::invalid_argument("page_size is too big"))?;
        let (users, total_count) = self
            .service
            .list_users(page_u32, page_size_u32)
            .await
            .map_err(map_err)?;
        Ok(Response::new(ListUsersResponse {
            total_count: total_count as u64,
            users: users.into_iter().map(|u| u.into()).collect(),
            page,
            page_size,
        }))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let UpdateUserRequest {
            user_uuid,
            username,
            password,
        } = request.into_inner();
        let id = parse_uuid!(user_uuid)?;
        let domain_req = DomainUpdateUserRequest {
            fields: crate::api::grpc::user::dto::UserFields {
                username,
                password,
                _is_active: None,
            },
        };
        let user = self
            .service
            .update_user(id, domain_req)
            .await
            .map_err(map_err)?;
        Ok(Response::new(user.into()))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let DeleteUserRequest { user_uuid } = request.into_inner();
        let id = parse_uuid!(user_uuid)?;
        self.service.deactivate_user(id).await.map_err(map_err)?;
        Ok(Response::new(DeleteUserResponse { success: true }))
    }
}

fn map_err(e: UserDomainError) -> Status {
    use UserDomainError as E;
    match e {
        E::Validation(msg) => Status::invalid_argument(msg),
        E::UserNotFound => Status::not_found("User not found"),
        E::Hashing(_) => Status::internal("Failed to hash password"),
        E::Repo(_) => Status::internal("Repository error"),
    }
}
