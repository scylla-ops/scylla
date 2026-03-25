use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use scylla_core::domain::errors::DomainError;

use super::response::ErrorBody;

pub struct AppError(DomainError);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            DomainError::NotFound { entity_type, id } => (
                StatusCode::NOT_FOUND,
                format!("{entity_type} with id '{id}' not found"),
            ),
            DomainError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            DomainError::BusinessRule(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            DomainError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            DomainError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            DomainError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            DomainError::Infrastructure(msg) => {
                tracing::error!("Infrastructure error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            DomainError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}

impl AppError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self(DomainError::Validation(msg.into()))
    }
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        Self(err)
    }
}
