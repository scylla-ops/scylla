use crate::api::grpc::user::models::User;
use crate::api::grpc::user::repos::UserRepository;
use crate::api::grpc::utils::verify_password;
use derive_more::Constructor;
use pasetors::claims::ClaimsValidationRules;
use pasetors::keys::{Generate, SymmetricKey};
use pasetors::token::UntrustedToken;
use pasetors::version4::V4;
use pasetors::{Local, local};
use serde::Deserialize;
use std::sync::LazyLock;
use surrealdb::RecordId;
use thiserror::Error;

static PASETO_SECRET: LazyLock<SymmetricKey<V4>> =
    LazyLock::new(|| SymmetricKey::<V4>::generate().unwrap());

#[derive(Constructor)]
pub struct AuthService<R: UserRepository> {
    _marker: std::marker::PhantomData<R>,
}

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
    _PasetoVerification(String),
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

#[derive(Deserialize, Debug)]
pub struct AuthedUser {
    pub _id: RecordId,
}

impl<R: UserRepository> AuthService<R> {
    fn generate_paseto(user: User) -> Result<String, AuthError> {
        use chrono::{Duration, Utc};
        use pasetors::claims::Claims;

        let expiration = Utc::now() + Duration::hours(1);
        let mut claims = Claims::new()?;

        claims.subject(&user.id.to_string())?;
        claims.issuer("auth-service")?;
        claims.issued_at(&Utc::now().to_rfc3339())?;
        claims.expiration(&expiration.to_rfc3339())?;

        claims.add_additional("username", user.username.to_string())?;

        Ok(local::encrypt(&PASETO_SECRET, &claims, None, None)?)
    }

    pub async fn _verify_paseto(token: &str) -> Result<AuthedUser, AuthError> {
        let mut validation_rules = ClaimsValidationRules::new();
        validation_rules.validate_issuer_with("auth-service");

        let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
            .map_err(|e| AuthError::_PasetoVerification(e.to_string()))?;

        let trusted_token = local::decrypt(
            &PASETO_SECRET,
            &untrusted_token,
            &validation_rules,
            None,
            None,
        )
        .map_err(|e| AuthError::_PasetoVerification(e.to_string()))?;

        let claims = trusted_token
            .payload_claims()
            .ok_or_else(|| AuthError::_PasetoVerification("No claims found in token".to_string()))?;

        let subject = claims
            .get_claim("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AuthError::_PasetoVerification("No valid subject claim found".to_string())
            })?;

        let user = R::get_user_by_id(subject.to_string())
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(AuthedUser { _id: user.id })
    }

    pub async fn login(username: String, password: String) -> Result<String, AuthError> {
        let user = match R::get_user_by_username(username).await? {
            Some(user) => user,
            None => return Err(AuthError::UserNotFound),
        };

        if !user.is_active {
            return Err(AuthError::AccountDisabled);
        }

        match verify_password(&password, &user.password_hash) {
            Ok(true) => (),
            _ => return Err(AuthError::IncorrectPassword),
        }

        Self::generate_paseto(user)
    }
}
