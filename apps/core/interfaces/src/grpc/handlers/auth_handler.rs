use crate::grpc::services::services::auth::*;
use derive_more::Constructor;
use std::sync::Arc;
use tonic::{Request, Response, Status};

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
            username: Username::try_from(req.username)?,
            password: Password::try_from(req.password)?,
        };

        let response = self.container.login_use_case().execute(dto).await?;

        Ok(Response::new(LoginResponse {
            token: response.token,
            user_id: response.user_id.to_string(),
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
            .await?;

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

        self.container.revoke_token_use_case().execute(dto).await?;

        Ok(Response::new(RevokeTokenResponse {}))
    }
}
