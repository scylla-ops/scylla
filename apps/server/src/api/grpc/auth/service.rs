use crate::api::grpc::auth::AuthService;
use crate::api::grpc::user::models::User;
use pasetors::local;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Utilisateur introuvable")]
    UserNotFound,
    #[error("Account is disabled")]
    AccountDisabled,
    #[error("Incorrect password")]
    IncorrectPassword,
    #[error("Paseto generation failed: {0}")]
    PasetoGeneration(String),
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

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

        let token = local::encrypt(&symmetric_key, &claims, None, None)?;

        Ok(token)
    }

    pub async fn login(&self, username: String, password: String) -> Result<String, AuthError> {
        let user = match self.repo.get_user_by_username(username).await? {
            Some(user) => user,
            None => return Err(AuthError::UserNotFound),
        };

        if !user.is_active {
            return Err(AuthError::AccountDisabled);
        }

        if !Self::verify_password(self, &password, &user.password_hash) {
            return Err(AuthError::IncorrectPassword);
        }

        let token = Self::generate_paseto(self, user)
            .map_err(|e| AuthError::PasetoGeneration(e.to_string()))?;

        Ok(token)
    }
}
