use crate::application::ports::HashService;
use crate::domain::errors::{DomainError, DomainResult};
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
    #[instrument(skip(self, password))]
    async fn hash(&self, password: &Password) -> DomainResult<PasswordHash> {
        let argon2 = self.argon2.clone();
        let password = password.clone();
        let hash = argon2
            .hash_password(password.as_str().as_bytes())
            .map_err(|e| DomainError::internal(format!("Failed to hash password: {e}")))?;

        PasswordHash::new(hash.to_string())
    }

    #[instrument(skip(self, password, hash))]
    async fn verify(&self, password: &Password, hash: &PasswordHash) -> DomainResult<bool> {
        let argon2 = self.argon2.clone();
        let password = password.clone();
        let hash = hash.as_str().to_string();
        let parsed_hash = Argon2PasswordHash::new(&hash)
            .map_err(|e| DomainError::internal(format!("Failed to parse hash: {e}")))?;

        Ok(argon2
            .verify_password(password.as_str().as_bytes(), &parsed_hash)
            .is_ok())
    }
}
