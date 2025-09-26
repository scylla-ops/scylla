use crate::api::grpc::auth::service::{AUTH_SERVICE, AuthError};
use derive_more::Constructor;
use protocol::services::{LoginRequest, LoginResponse, auth_service_server};
use protocol::tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct AuthController {}

#[async_trait::async_trait]
impl auth_service_server::AuthService for AuthController {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        let res = AUTH_SERVICE
            .login(req.username, req.password)
            .await
            .map_err(|e| match e {
                AuthError::UserNotFound => Status::not_found("User not found"),
                AuthError::AccountDisabled => Status::permission_denied("Account is disabled"),
                AuthError::IncorrectPassword => Status::permission_denied("Incorrect password"),
                AuthError::PasetoGeneration(_) => Status::internal("Error generating token"),
                AuthError::Repo(e) => {
                    tracing::error!("Authentication error (repo): {}", e);
                    Status::internal("Server error")
                }
                AuthError::PasetoVerification(e) => {
                    tracing::error!("Authentication error (paseto): {}", e);
                    Status::internal("Server error")
                }
            })?;

        Ok(Response::new(LoginResponse { token: res }))
    }
}
