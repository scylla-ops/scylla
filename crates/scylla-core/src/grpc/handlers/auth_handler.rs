use crate::application::AuthUseCases;
use crate::grpc::adapter::run_public;
use crate::grpc::convert::wrap;
use derive_more::Constructor;
use scylla_extension::Actions;
use scylla_proto::auth::v1::{
    LoginRequest, LoginResponse, RevokeTokenRequest, RevokeTokenResponse, ValidateTokenRequest,
    ValidateTokenResponse, auth_service_server::AuthService, validate_token_response,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct AuthHandler {
    actions: Arc<Actions>,
    sessions: Arc<AuthUseCases>,
}

#[async_trait::async_trait]
impl AuthService for AuthHandler {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let session = run_public(&self.actions, &*self.sessions, request).await?;
        Ok(Response::new(LoginResponse {
            token: session.token().to_string(),
            user_id: wrap(session.user_id().to_string()),
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let result = if run_public(&self.actions, &*self.sessions, request).await? {
            validate_token_response::Result::Valid(validate_token_response::Valid {})
        } else {
            validate_token_response::Result::Invalid(validate_token_response::Invalid {})
        };
        Ok(Response::new(ValidateTokenResponse {
            result: Some(result),
        }))
    }

    async fn revoke_token(
        &self,
        request: Request<RevokeTokenRequest>,
    ) -> Result<Response<RevokeTokenResponse>, Status> {
        run_public(&self.actions, &*self.sessions, request).await?;
        Ok(Response::new(RevokeTokenResponse {}))
    }
}
