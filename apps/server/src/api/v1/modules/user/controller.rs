use crate::api::v1::common::extractors::validated_json::ValidatedJson;
use crate::api::v1::common::responses::helper::ApiResponse;
use crate::api::v1::modules::user::dto::{NewUserRequest, UpdateUserRequest};
use crate::api::v1::modules::user::repository::UserRepository;
use crate::api::v1::modules::user::service::{UserService, UserServiceTrait};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use std::sync::Arc;
use uuid::Uuid;

type UserState = State<Arc<UserService<UserRepository>>>;

pub struct UserController {}

impl UserController {
    // Create a new user
    // todo should be admin only request
    pub async fn create_user(
        State(service): UserState,
        ValidatedJson(req): ValidatedJson<NewUserRequest>,
    ) -> impl IntoResponse {
        match service.create_user(req).await {
            Ok(_) => {
                ApiResponse::success(axum::http::StatusCode::CREATED, "User created successfully")
            }
            Err(e) => {
                tracing::error!("Failed to create user: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create user: {e}"),
                )
            }
        }
    }

    // Get user by ID
    pub async fn get_user_by_id(
        State(service): UserState,
        user_uuid: Path<Uuid>,
    ) -> impl IntoResponse {
        match service.get_user_by_id(*user_uuid).await {
            Ok(Some(user)) => ApiResponse::success(axum::http::StatusCode::OK, user),
            Ok(None) => ApiResponse::error(
                axum::http::StatusCode::NOT_FOUND,
                "User not found".to_string(),
            ),
            Err(e) => {
                tracing::error!("Failed to fetch user: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to fetch user: {e}"),
                )
            }
        }
    }

    // Get all users
    pub async fn get_all_users(State(service): UserState) -> impl IntoResponse {
        match service.get_all_users().await {
            Ok(users) => ApiResponse::success(axum::http::StatusCode::OK, users),
            Err(e) => {
                tracing::error!("Failed to fetch users: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to fetch users: {e}"),
                )
            }
        }
    }

    // Update user by ID
    pub async fn update_user_by_id(
        State(service): UserState,
        user_uuid: Path<Uuid>,
        ValidatedJson(req): ValidatedJson<UpdateUserRequest>,
    ) -> impl IntoResponse {
        match service.update_user_by_id(*user_uuid, req).await {
            Ok(_) => ApiResponse::success(axum::http::StatusCode::OK, "User updated successfully"),
            Err(e) => {
                tracing::error!("Failed to update user: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to update user: {e}"),
                )
            }
        }
    }

    // Deactivate user by ID
    pub async fn deactivate_user_by_id(
        State(service): UserState,
        user_uuid: Path<Uuid>,
    ) -> impl IntoResponse {
        match service.deactivate_user_by_id(*user_uuid).await {
            Ok(_) => {
                ApiResponse::success(axum::http::StatusCode::OK, "User deactivated successfully")
            }
            Err(e) => {
                tracing::error!("Failed to deactivate user: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to deactivate user: {e}"),
                )
            }
        }
    }
}
