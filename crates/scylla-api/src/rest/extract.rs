use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use scylla_core::application::SessionRepository;
use scylla_core::domain::entities::UserId;

use super::AppState;
use super::error::AppError;

pub struct Auth(pub UserId);

impl FromRequestParts<AppState> for Auth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(parts)?;

        let session = state.session_repo.find_by_token(&token).await?;

        if session.is_expired() {
            let _ = state.session_repo.delete_by_token(&token).await;
            return Err(scylla_core::domain::errors::DomainError::unauthorized(
                "Token has expired",
            )
            .into());
        }

        Ok(Auth(session.user_id().clone()))
    }
}

fn extract_bearer_token(parts: &Parts) -> Result<String, AppError> {
    let header = parts
        .headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            scylla_core::domain::errors::DomainError::unauthorized("Missing authorization header")
        })?;

    let token = header.strip_prefix("Bearer ").ok_or_else(|| {
        scylla_core::domain::errors::DomainError::unauthorized(
            "Invalid authorization header format",
        )
    })?;

    Ok(token.to_string())
}
