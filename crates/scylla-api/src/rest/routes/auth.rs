use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use scylla_core::domain::value_objects::user::{Password, Username};

use crate::rest::AppState;
use crate::rest::error::AppError;
use crate::rest::extract::Auth;
use crate::rest::response::{
    ErrorBody, LoginRequest, LoginResponse, TokenRequest, ValidateTokenResponse,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/validate", post(validate_token))
        .route("/revoke", post(revoke_token))
}

#[utoipa::path(
    post, path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Invalid credentials", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
    )
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let username = Username::new(&body.username).map_err(AppError::from)?;
    let password = Password::new(&body.password).map_err(AppError::from)?;

    let (token, user_id) = state.auth_uc.login(username, password).await?;

    Ok(Json(LoginResponse {
        token,
        user_id: user_id.to_string(),
    }))
}

#[utoipa::path(
    post, path = "/api/v1/auth/validate",
    tag = "Auth",
    request_body = TokenRequest,
    responses(
        (status = 200, description = "Token validation result", body = ValidateTokenResponse),
    )
)]
pub(crate) async fn validate_token(
    State(state): State<AppState>,
    Json(body): Json<TokenRequest>,
) -> Result<Json<ValidateTokenResponse>, AppError> {
    let is_valid = state.auth_uc.validate_token(&body.token).await?;
    Ok(Json(ValidateTokenResponse { is_valid }))
}

#[utoipa::path(
    post, path = "/api/v1/auth/revoke",
    tag = "Auth",
    request_body = TokenRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Token revoked"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
    )
)]
pub(crate) async fn revoke_token(
    State(state): State<AppState>,
    Auth(_user_id): Auth,
    Json(body): Json<TokenRequest>,
) -> Result<Json<()>, AppError> {
    state.auth_uc.revoke_token(&body.token).await?;
    Ok(Json(()))
}
