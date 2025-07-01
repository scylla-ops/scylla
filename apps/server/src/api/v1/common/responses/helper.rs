use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_json::{Value, json};

pub enum ApiResponse {
    Success(StatusCode, Value),
    Error(StatusCode, String),
    #[allow(dead_code)]
    Empty,
}

impl ApiResponse {
    pub fn success<T: Serialize>(status: StatusCode, data: T) -> Self {
        ApiResponse::Success(status, serde_json::to_value(data).unwrap_or(json!(null)))
    }

    pub fn error(status: StatusCode, message: impl Into<String>) -> Self {
        ApiResponse::Error(status, message.into())
    }

    #[allow(dead_code)]
    pub fn empty() -> Self {
        ApiResponse::Empty
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiResponse::Success(status, data) => {
                (status, Json(json!({ "data": data }))).into_response()
            }
            ApiResponse::Error(status, msg) => {
                (status, Json(json!({ "error": msg }))).into_response()
            }
            ApiResponse::Empty => StatusCode::NO_CONTENT.into_response(),
        }
    }
}
