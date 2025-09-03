use crate::api::grpc::auth::AuthService;
use crate::api::v1::models::users::User;
use pasetors::local;
use protocol::services::{LoginRequest, LoginResponse, auth_service_server};
use protocol::tonic;
use protocol::tonic::{Request, Response, Status};
use tracing::log::debug;

impl AuthService {
    fn verify_password(&self, password: &str, password_hash: &str) -> bool {
        bcrypt::verify(password.as_bytes(), password_hash).unwrap_or(false)
    }

    fn generate_paseto(&self, user: User) -> Result<String, Box<dyn std::error::Error>> {
        use chrono::{Duration, Utc};
        use pasetors::claims::Claims;
        use pasetors::keys::SymmetricKey;
        let symmetric_key = SymmetricKey::from(self.paseto_secret.as_bytes())?;

        let expiration = Utc::now() + Duration::hours(1);
        let mut claims = Claims::new()?;

        claims.subject(&user.id.to_string())?;
        claims.issued_at(&Utc::now().to_rfc3339())?;
        claims.expiration(&expiration.to_rfc3339())?;

        claims.add_additional("username", user.username)?;

        debug!("{:?}", claims);

        let token = local::encrypt(&symmetric_key, &claims, None, None)?;

        Ok(token)
    }
}

#[tonic::async_trait]
impl auth_service_server::AuthService for AuthService {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        // 1. Rechercher l'utilisateur par username
        let user = match self.repo.get_user_by_username(req.username).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(Status::not_found("Utilisateur introuvable")),
            Err(e) => {
                tracing::error!("Erreur lors de la recherche de l'utilisateur: {}", e);
                return Err(Status::internal("Erreur serveur"));
            }
        };

        // 2. Vérifier si l'utilisateur est actif
        if !user.is_active {
            return Err(Status::permission_denied("Account is disabled"));
        }

        // 3. Vérifier le mot de passe
        if !AuthService::verify_password(self, &req.password, &user.password_hash) {
            return Err(Status::permission_denied("Incorrect password"));
        }

        // 4. Generate Paseto
        let token = match AuthService::generate_paseto(self, user) {
            Ok(token) => token,
            Err(e) => {
                tracing::error!("Erreur lors de la génération du token: {}", e);
                return Err(Status::internal("Erreur serveur"));
            }
        };

        Ok(Response::new(LoginResponse { token }))
    }
}
