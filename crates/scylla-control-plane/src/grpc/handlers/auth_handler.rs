use crate::application::{AuthUseCases, HashService, SessionRepository, UserRepository};
use crate::grpc::convert::wrap;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::domain::ids::UserId;
use scylla_core::domain::user::Password;
use scylla_protocol::auth::v1::{
    LoginRequest, LoginResponse, RevokeTokenRequest, RevokeTokenResponse, ValidateTokenRequest,
    ValidateTokenResponse, auth_service_server::AuthService, validate_token_response,
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

        let password = Password::new(&req.password).map_err(domain_error_to_status)?;

        let (token, user_id) = self
            .use_cases
            .login(req.identifier, password)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(LoginResponse {
            token,
            user_id: wrap(user_id.to_string()),
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        let is_valid = if req.token.is_empty() {
            false
        } else {
            self.use_cases
                .validate_token(&req.token)
                .await
                .map_err(domain_error_to_status)?
        };

        Ok(Response::new(ValidateTokenResponse {
            result: Some(if is_valid {
                validate_token_response::Result::Valid(validate_token_response::Valid {})
            } else {
                validate_token_response::Result::Invalid(validate_token_response::Invalid {})
            }),
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
