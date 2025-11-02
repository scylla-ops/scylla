use crate::application::ports::PasswordHasher;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::Password;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
};
use async_trait::async_trait;
use getrandom::getrandom;

pub struct Argon2PasswordHasher {
    argon2: Argon2<'static>,
}

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, password: &Password) -> DomainResult<String> {
        // generate cryptographically secure random salt (16 bytes)
        let mut salt_bytes = [0u8; 16];
        getrandom(&mut salt_bytes)
            .map_err(|e| DomainError::internal(format!("Failed to generate random salt: {}", e)))?;
        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| DomainError::internal(format!("Failed to encode salt: {}", e)))?;

        let hash = self
            .argon2
            .hash_password(password.as_str().as_bytes(), &salt)
            .map_err(|e| DomainError::internal(format!("Failed to hash password: {}", e)))?;

        Ok(hash.to_string())
    }

    async fn verify(&self, password: &Password, hash: &str) -> DomainResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| DomainError::internal(format!("Failed to parse hash: {}", e)))?;

        Ok(self
            .argon2
            .verify_password(password.as_str().as_bytes(), &parsed_hash)
            .is_ok())
    }
}
