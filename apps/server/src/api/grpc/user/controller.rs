use crate::api::grpc::user::models::{CreateUserInput, UpdateUserInput};
#[cfg(feature = "surreal")]
use crate::api::grpc::user::repos::surreal::UserRepositorySurreal;
use crate::api::grpc::user::service::UserDomainError;
use crate::api::grpc::user::service::UserService;
use protocol::{
    services::{
        CreateUserRequest, DeleteUserRequest, DeleteUserResponse, GetUserRequest, ListUsersRequest,
        ListUsersResponse, UpdateUserRequest, UserResponse, user_service_server,
    },
    tonic::{Request, Response, Status},
};

#[cfg(feature = "surreal")]
type UserRepo = UserRepositorySurreal;

pub struct UserController;

#[async_trait::async_trait]
impl user_service_server::UserService for UserController {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let CreateUserRequest { username, password } = request.into_inner();
        let domain_req: CreateUserInput = CreateUserInput { username, password };
        let user = UserService::<UserRepo>::create_user(domain_req)
            .await
            .map_err(map_err)?;
        Ok(Response::new(UserResponse {
            user_id: user.id.key().to_string(),
            username: user.username.to_string(),
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
            is_active: user.is_active,
        }))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let GetUserRequest { user_id } = request.into_inner();
        let user = UserService::<UserRepo>::get_user(user_id)
            .await
            .map_err(map_err)?;
        Ok(Response::new(UserResponse {
            user_id: user.id.key().to_string(),
            username: user.username.to_string(),
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
            is_active: user.is_active,
        }))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        /*let token = extract_bearer_token(&request)?;

        let user = AUTH_SERVICE.verify_paseto(&token).await.map_err(|e| {
            debug!("Failed to verify token: {:?}", e);
            Status::unauthenticated("Invalid token")
        })?;

        debug!("{:?}", user);
        */

        let ListUsersRequest { page, page_size } = request.into_inner();
        let page_u32 =
            u32::try_from(page).map_err(|_| Status::invalid_argument("page is too big"))?;
        let page_size_u32 = u32::try_from(page_size)
            .map_err(|_| Status::invalid_argument("page_size is too big"))?;
        let (users, total_count) = UserService::<UserRepo>::list_users(page_u32, page_size_u32)
            .await
            .map_err(map_err)?;
        Ok(Response::new(ListUsersResponse {
            total_count: total_count as u64,
            users: users
                .into_iter()
                .map(|u| UserResponse {
                    user_id: u.id.key().to_string(),
                    username: u.username.to_string(),
                    created_at: u.created_at.to_rfc3339(),
                    updated_at: u.updated_at.to_rfc3339(),
                    is_active: u.is_active,
                })
                .collect(),
            page,
            page_size,
        }))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let UpdateUserRequest {
            user_id,
            username,
            password,
            is_active,
        } = request.into_inner();
        let domain_req = UpdateUserInput {
            username,
            password,
            is_active,
        };
        let user = UserService::<UserRepo>::update_user(user_id, domain_req)
            .await
            .map_err(map_err)?;
        Ok(Response::new(UserResponse {
            user_id: user.id.key().to_string(),
            username: user.username.to_string(),
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
            is_active: user.is_active,
        }))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let DeleteUserRequest { user_id } = request.into_inner();
        UserService::<UserRepo>::deactivate_user(user_id)
            .await
            .map_err(map_err)?;
        Ok(Response::new(DeleteUserResponse::default()))
    }
}

fn map_err(e: UserDomainError) -> Status {
    use UserDomainError as E;
    match e {
        E::Validation(msg) => Status::invalid_argument(msg),
        E::InvalidUsername(e) => Status::invalid_argument(e.to_string()),
        E::InvalidPassword(msg) => Status::invalid_argument(msg),
        E::InvalidPagination { field } => {
            Status::invalid_argument(format!("invalid pagination parameter: {}", field))
        }
        E::UserNotFound => Status::not_found("User not found"),
        E::Hashing(_) => Status::internal("Failed to hash password"),
        E::Repo(e) => Status::internal(format!("Repository error: {}", e)),
    }
}
