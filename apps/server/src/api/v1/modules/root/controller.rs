use crate::api::v1::common::responses::helper::ApiResponse;
use axum::http::StatusCode;
use axum::response::IntoResponse;

// Root controller for handling root-level HTTP requests
pub struct RootController;

impl RootController {
    pub async fn fallback() -> impl IntoResponse {
        ApiResponse::error(StatusCode::NOT_FOUND, "API endpoint not found".to_string())
    }
}
