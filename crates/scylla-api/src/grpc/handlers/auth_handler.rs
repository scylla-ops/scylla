use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::AuthUseCases;
use scylla_core::application::{HashService, SessionRepository, UserRepository};
use scylla_core::domain::entities::UserId;
use scylla_core::domain::value_objects::user::{Password, Username};
use scylla_protocol::services::auth::{
    LoginRequest, LoginResponse, RevokeTokenRequest, RevokeTokenResponse, ValidateTokenRequest,
    ValidateTokenResponse, auth_service_server::AuthService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct AuthHandler<U: UserRepository, S: SessionRepository, H: HashService> {
    use_cases: Arc<AuthUseCases<U, S, H>>,
}

#[async_trait::async_trait]
impl<
    U: UserRepository + Send + Sync + 'static,
    S: SessionRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
> AuthService for AuthHandler<U, S, H>
{
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        let username = Username::new(&req.username).map_err(domain_error_to_status)?;
        let password = Password::new(&req.password).map_err(domain_error_to_status)?;

        let (token, user_id) = self
            .use_cases
            .login(username, password)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(LoginResponse {
            token,
            user_id: user_id.to_string(),
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        if req.token.is_empty() {
            return Ok(Response::new(ValidateTokenResponse {
                is_valid: Some(false),
            }));
        }

        let is_valid = self
            .use_cases
            .validate_token(&req.token)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ValidateTokenResponse {
            is_valid: Some(is_valid),
        }))
    }

    async fn revoke_token(
        &self,
        request: Request<RevokeTokenRequest>,
    ) -> Result<Response<RevokeTokenResponse>, Status> {
        let req = request.into_inner();

        if req.token.is_empty() {
            return Err(Status::invalid_argument("Token cannot be empty"));
        }

        self.use_cases
            .revoke_token(&req.token)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RevokeTokenResponse {}))
    }
}

impl<U: UserRepository, S: SessionRepository, H: HashService> AuthHandler<U, S, H> {
    pub async fn get_user_id_from_token(&self, token: &str) -> Result<UserId, Status> {
        if token.is_empty() {
            return Err(Status::unauthenticated("Token cannot be empty"));
        }
        self.use_cases
            .get_user_id_from_token(token)
            .await
            .map_err(domain_error_to_status)
    }

    pub async fn touch_session(&self, token: &str) -> Result<(), Status> {
        self.use_cases
            .touch_session(token)
            .await
            .map_err(domain_error_to_status)
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u64, Status> {
        self.use_cases
            .cleanup_expired()
            .await
            .map_err(domain_error_to_status)
    }
}
