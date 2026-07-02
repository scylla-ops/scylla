use crate::application::HashService;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::app::{AppSecret, AppSecretHash};
use crate::domain::value_objects::user::{Password, PasswordHash};
use argon2::{
    Argon2,
    password_hash::phc::PasswordHash as Argon2PasswordHash,
    password_hash::{PasswordHasher as _, PasswordVerifier},
};
use async_trait::async_trait;
use tracing::instrument;

#[derive(Default)]
pub struct Argon2HashService {
    argon2: Argon2<'static>,
}

impl Argon2HashService {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl HashService for Argon2HashService {
    // Argon2 is deliberately CPU-heavy (tens of ms). Running it inline would
    // block an async worker thread for that whole time, so each hash/verify is
    // moved onto Tokio's blocking pool — which is exactly what the per-call
    // `clone()`s (of the hasher and the input) are for.
    #[instrument(skip(self, password))]
    async fn hash(&self, password: &Password) -> DomainResult<PasswordHash> {
        let argon2 = self.argon2.clone();
        let password = password.clone();
        let hash = tokio::task::spawn_blocking(move || {
            argon2
                .hash_password(password.as_str().as_bytes())
                .map(|h| h.to_string())
                .map_err(|e| DomainError::internal(format!("Failed to hash password: {e}")))
        })
        .await
        .map_err(|e| DomainError::internal(format!("password hashing task failed: {e}")))??;

        PasswordHash::new(hash)
    }

    #[instrument(skip(self, password, hash))]
    async fn verify(&self, password: &Password, hash: &PasswordHash) -> DomainResult<bool> {
        let argon2 = self.argon2.clone();
        let password = password.clone();
        let hash = hash.as_str().to_string();
        tokio::task::spawn_blocking(move || {
            let parsed_hash = Argon2PasswordHash::new(&hash)
                .map_err(|e| DomainError::internal(format!("Failed to parse hash: {e}")))?;
            Ok(argon2
                .verify_password(password.as_str().as_bytes(), &parsed_hash)
                .is_ok())
        })
        .await
        .map_err(|e| DomainError::internal(format!("password verification task failed: {e}")))?
    }

    #[instrument(skip(self, secret))]
    async fn hash_secret(&self, secret: &AppSecret) -> DomainResult<AppSecretHash> {
        let argon2 = self.argon2.clone();
        let secret = secret.clone();
        let hash = tokio::task::spawn_blocking(move || {
            argon2
                .hash_password(secret.as_str().as_bytes())
                .map(|h| h.to_string())
                .map_err(|e| DomainError::internal(format!("Failed to hash app secret: {e}")))
        })
        .await
        .map_err(|e| DomainError::internal(format!("app-secret hashing task failed: {e}")))??;

        AppSecretHash::new(hash)
    }

    #[instrument(skip(self, secret, hash))]
    async fn verify_secret(&self, secret: &AppSecret, hash: &AppSecretHash) -> DomainResult<bool> {
        let argon2 = self.argon2.clone();
        let secret = secret.clone();
        let hash = hash.as_str().to_string();
        tokio::task::spawn_blocking(move || {
            let parsed_hash = Argon2PasswordHash::new(&hash)
                .map_err(|e| DomainError::internal(format!("Failed to parse hash: {e}")))?;
            Ok(argon2
                .verify_password(secret.as_str().as_bytes(), &parsed_hash)
                .is_ok())
        })
        .await
        .map_err(|e| DomainError::internal(format!("app-secret verification task failed: {e}")))?
    }
}
