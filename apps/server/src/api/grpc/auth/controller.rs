use crate::api::grpc::auth::AuthService;
use derive_more::Constructor;
use protocol::services::{LoginRequest, LoginResponse, auth_service_server};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

#[derive(Constructor)]
pub struct AuthController {
    service: Arc<AuthService>,
}

#[async_trait::async_trait]
impl auth_service_server::AuthService for AuthController {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        let res = self
            .service
            .login(req.username, req.password)
            .await
            .map_err(|e| match e {
                crate::api::grpc::auth::service::AuthError::UserNotFound => {
                    Status::not_found("Utilisateur introuvable")
                }
                crate::api::grpc::auth::service::AuthError::AccountDisabled => {
                    Status::permission_denied("Account is disabled")
                }
                crate::api::grpc::auth::service::AuthError::IncorrectPassword => {
                    Status::permission_denied("Incorrect password")
                }
                crate::api::grpc::auth::service::AuthError::PasetoGeneration(_) => {
                    Status::internal("Erreur lors de la génération du token")
                }
                crate::api::grpc::auth::service::AuthError::Repo(e) => {
                    tracing::error!("Erreur lors de l'authentification (repo): {}", e);
                    Status::internal("Erreur serveur")
                }
            })?;

        Ok(Response::new(LoginResponse { token: res }))
    }
}
