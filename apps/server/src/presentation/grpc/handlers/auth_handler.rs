use crate::application::dto::{LoginRequestDto, RevokeTokenRequestDto, ValidateTokenRequestDto};
use crate::domain::value_objects::{Password, Username};
use crate::presentation::grpc::mappers::map_domain_error_to_status;
use crate::shared::di::AppContainer;
use derive_more::Constructor;
use protocol::services::auth::{
    LoginRequest, LoginResponse, RevokeTokenRequest, RevokeTokenResponse, ValidateTokenRequest,
    ValidateTokenResponse, auth_service_server,
};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

#[derive(Constructor)]
pub struct AuthHandler {
    container: Arc<AppContainer>,
}

#[async_trait::async_trait]
impl auth_service_server::AuthService for AuthHandler {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        let dto = LoginRequestDto {
            username: Username::new(req.username).map_err(map_domain_error_to_status)?,
            password: Password::new(req.password).map_err(map_domain_error_to_status)?,
        };

        let response = self
            .container
            .login_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(LoginResponse {
            token: response.token,
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        let dto = ValidateTokenRequestDto { token: req.token };

        let response = self
            .container
            .validate_token_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(ValidateTokenResponse {
            is_valid: response.is_valid,
        }))
    }

    async fn revoke_token(
        &self,
        request: Request<RevokeTokenRequest>,
    ) -> Result<Response<RevokeTokenResponse>, Status> {
        let req = request.into_inner();

        let dto = RevokeTokenRequestDto { token: req.token };

        self.container
            .revoke_token_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(RevokeTokenResponse {}))
    }
}
