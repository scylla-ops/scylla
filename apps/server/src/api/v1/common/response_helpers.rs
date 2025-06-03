use crate::api::v1::dto::ApiResponse;
use axum::Json;
use axum::http::StatusCode;

/// Helper function to create a success response
pub fn success<T>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse::success(data))
}

/// Helper function to create an error response
pub fn error<T>(message: impl Into<String>) -> Json<ApiResponse<T>> {
    Json(ApiResponse::error(message.into()))
}

/// Helper function to create a validation error response
pub fn validation_error<T>(e: impl std::fmt::Display) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::BAD_REQUEST,
        error(format!("Validation error: {}", e)),
    )
}

/// Helper function to create a not found error response
pub fn not_found<T>(resource: impl std::fmt::Display) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::NOT_FOUND,
        error(format!("Not found: {}", resource)),
    )
}

/// Helper function to create an internal server error response
pub fn internal_error<T>(e: impl std::fmt::Display) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        error(format!("Internal server error: {}", e)),
    )
}
