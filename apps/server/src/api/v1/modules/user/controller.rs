use crate::api::v1::common::extractors::validated_json::ValidatedJson;
use crate::api::v1::common::responses::helper::ApiResponse;
use crate::api::v1::modules::user::dto::NewUserRequest;
use crate::api::v1::modules::user::service::UserService;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use std::sync::Arc;
use uuid::Uuid;

pub struct UserController {}

impl UserController {
    // Create a new user
    // todo should be admin only request
    pub async fn create_user(
        State(service): State<Arc<UserService>>,
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
        State(service): State<Arc<UserService>>,
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
}
