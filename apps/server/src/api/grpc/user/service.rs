use crate::api::grpc::user::dto::{NewUser, NewUserRequest, UpdateUser, UserFields};
use crate::api::grpc::user::service::tonic::{Response, Status};
use crate::api::grpc::user::{UserService, dto};
use bcrypt::BcryptError;
use chrono::Utc;
use protocol::services::{
    CreateUserRequest, DeleteUserRequest, DeleteUserResponse, GetUserRequest, ListUsersRequest,
    ListUsersResponse, UpdateUserRequest, UserResponse, user_service_server,
};
use protocol::tonic;
use tonic::Request;
use tracing::{error, warn};
use uuid::Uuid;
use validator::Validate;

impl UserService {
    fn hash_password(password: &str) -> Result<String, BcryptError> {
        //todo: ne pas utiliser le coût par défaut en production
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
    }
}

#[tonic::async_trait]
impl user_service_server::UserService for UserService {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req: NewUserRequest = request.into_inner().into();

        if let Err(e) = req.validate() {
            return Err(Status::invalid_argument(format!(
                "Failed to validate request: {}",
                e
            )));
        }

        let password_hash = Self::hash_password(&req.fields.password.unwrap())
            .map_err(|e| Status::internal(format!("Failed to hash password: {}", e)))?;

        warn!("{:?}", password_hash);

        let new_user = NewUser {
            username: req.fields.username.unwrap(),
            password_hash,
        };

        match self.repo.create_user(new_user).await {
            Ok(user) => Ok(Response::new(user.into())),
            Err(err) => {
                error!("Error creating user: {}", err);
                //todo handle diesel errors
                Err(Status::internal(format!("Error creating user: {}", err)))
            }
        }
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let user_uuid: Uuid = request
            .into_inner()
            .user_uuid
            .parse()
            .map_err(|e| Status::invalid_argument(format!("Invalid user_uuid: {}", e)))?;

        match self.repo.get_user_by_uuid(user_uuid).await {
            Ok(Some(user)) => Ok(Response::new(user.into())),
            Ok(None) => Err(Status::not_found("User not found")),
            Err(err) => {
                error!("Error getting user: {}", err);
                Err(Status::internal(format!("Error getting user: {}", err)))
            }
        }
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let ListUsersRequest { page, page_size } = request.into_inner();

        if page == 0 {
            return Err(Status::invalid_argument("page must be >= 1"));
        }
        if page_size == 0 {
            return Err(Status::invalid_argument("page_size must be >= 1"));
        }

        let limit_i64 = i64::try_from(page_size)
            .map_err(|_| Status::invalid_argument("page_size is too big"))?;

        let offset_u128 = (page as u128)
            .saturating_sub(1)
            .saturating_mul(page_size as u128);
        let offset_i64 = i64::try_from(offset_u128)
            .map_err(|_| Status::invalid_argument("computed offset is too big"))?;

        match self.repo.list_users(limit_i64, offset_i64).await {
            Ok(user_list) => Ok(Response::new(ListUsersResponse {
                total_count: user_list.len() as u64,
                users: user_list.into_iter().map(|u| u.into()).collect(),
                page,
                page_size,
            })),
            Err(err) => Err(Status::internal(format!("Error listing users: {}", err))),
        }
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

        let user_uuid: Uuid = user_uuid
            .parse()
            .map_err(|e| Status::invalid_argument(format!("Invalid user_uuid: {}", e)))?;

        let req: dto::UpdateUserRequest = dto::UpdateUserRequest {
            fields: UserFields {
                username,
                password,
                is_active: None,
            },
        };

        if let Err(e) = req.validate() {
            return Err(Status::invalid_argument(format!(
                "Failed to validate request: {}",
                e
            )));
        }

        let UserFields {
            username, password, ..
        } = req.fields;
        let password_hash = if let Some(ref pwd) = password {
            Some(
                Self::hash_password(pwd)
                    .map_err(|e| Status::internal(format!("Failed to hash password: {}", e)))?,
            )
        } else {
            None
        };

        let update_user = UpdateUser {
            username,
            password_hash,
            updated_at: Utc::now().naive_utc(),
        };

        match self.repo.update_user(user_uuid, update_user).await {
            Ok(Some(user)) => Ok(Response::new(user.into())),
            Ok(None) => Err(Status::not_found("User not found")),
            Err(err) => {
                error!("Error updating user: {}", err);
                Err(Status::internal(format!("Error updating user: {}", err)))
            }
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let DeleteUserRequest { user_uuid } = request.into_inner();
        let user_uuid: Uuid = user_uuid
            .parse()
            .map_err(|e| Status::invalid_argument(format!("Invalid user_uuid: {}", e)))?;

        match self.repo.deactivate_user(user_uuid).await {
            Ok(Some(_)) => Ok(Response::new(DeleteUserResponse { success: true })),
            Ok(None) => Err(Status::not_found("User not found")),
            Err(_) => Err(Status::internal("Error deleting user")),
        }
    }
}
