use crate::api::grpc::user::UserRepository;
use crate::api::grpc::user::models::User;
use crate::api::grpc::user::repo::UserRepositoryDiesel;
use crate::api::grpc::user::service::USER_SERVICE;
use crate::database::get_existing_db;
use derive_more::Constructor;
use pasetors::claims::ClaimsValidationRules;
use pasetors::keys::{Generate, SymmetricKey};
use pasetors::token::UntrustedToken;
use pasetors::version4::V4;
use pasetors::{Local, local};
use serde::Deserialize;
use std::sync::{Arc, LazyLock};
use thiserror::Error;
use uuid::Uuid;

#[derive(Constructor)]
pub struct AuthService {
    pub(crate) repo: Arc<dyn UserRepository>,
    pub(crate) paseto_secret: SymmetricKey<V4>,
}

pub static AUTH_SERVICE: LazyLock<Arc<AuthService>> = LazyLock::new(|| {
    let diesel_db = get_existing_db();

    Arc::new(AuthService::new(
        Arc::new(UserRepositoryDiesel::new(diesel_db.clone())),
        SymmetricKey::generate().expect("Unable to generate paseto secret"),
    ))
});

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Utilisateur introuvable")]
    UserNotFound,
    #[error("Account is disabled")]
    AccountDisabled,
    #[error("Incorrect password")]
    IncorrectPassword,
    #[error("Paseto generation failed: {0}")]
    PasetoGeneration(#[from] pasetors::errors::Error),
    #[error("Paseto verification failed: {0}")]
    PasetoVerification(String),
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

#[derive(Deserialize, Debug)]
pub struct AuthedUser {
    pub id: Uuid,
}

impl AuthService {
    fn verify_password(&self, password: &str, password_hash: &str) -> bool {
        bcrypt::verify(password.as_bytes(), password_hash).unwrap_or(false)
    }

    fn generate_paseto(&self, user: User) -> Result<String, AuthError> {
        use chrono::{Duration, Utc};
        use pasetors::claims::Claims;
        use pasetors::keys::SymmetricKey;
        let symmetric_key = SymmetricKey::from(self.paseto_secret.as_bytes())?;

        let expiration = Utc::now() + Duration::hours(1);
        let mut claims = Claims::new()?;

        claims.subject(&user.id.to_string())?;
        claims.issuer("auth-service")?;
        claims.issued_at(&Utc::now().to_rfc3339())?;
        claims.expiration(&expiration.to_rfc3339())?;

        claims.add_additional("username", user.username)?;

        let token = local::encrypt(&symmetric_key, &claims, None, None)?;

        Ok(token)
    }

    pub async fn verify_paseto(&self, token: &str) -> Result<AuthedUser, AuthError> {
        let mut validation_rules = ClaimsValidationRules::new();
        validation_rules.validate_issuer_with("auth-service");

        let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
            .map_err(|e| AuthError::PasetoVerification(e.to_string()))?;

        let trusted_token = local::decrypt(
            &self.paseto_secret,
            &untrusted_token,
            &validation_rules,
            None,
            None,
        )
        .map_err(|e| AuthError::PasetoVerification(e.to_string()))?;

        let claims = trusted_token
            .payload_claims()
            .ok_or_else(|| AuthError::PasetoVerification("No claims found in token".to_string()))?;

        let subject = claims
            .get_claim("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AuthError::PasetoVerification("No valid subject claim found".to_string())
            })?;

        let id =
            Uuid::parse_str(subject).map_err(|e| AuthError::PasetoVerification(e.to_string()))?;

        let user = USER_SERVICE
            .get_user(id)
            .await
            .map_err(|e| AuthError::Repo(e.into()))?;

        Ok(AuthedUser { id: user.id })
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

        let token = Self::generate_paseto(self, user)?;

        Ok(token)
    }
}
