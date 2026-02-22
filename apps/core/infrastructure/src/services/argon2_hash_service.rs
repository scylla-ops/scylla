use argon2::{
    Argon2,
    password_hash::phc::PasswordHash,
    password_hash::{PasswordHasher as _, PasswordVerifier},
};
use domain::errors::{DomainError, DomainResult};
use domain::ports::HashService;
use domain::value_objects::user::Password;
use async_trait::async_trait;

#[derive(Default)]
pub struct Argon2HashService {
    argon2: Argon2<'static>,
}

impl Argon2HashService {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl HashService for Argon2HashService {
    async fn hash(&self, password: &Password) -> DomainResult<String> {
        let argon2 = self.argon2.clone();
        let password = password.clone();
            let hash = argon2
                .hash_password(password.as_str().as_bytes())
                .map_err(|e| DomainError::internal(format!("Failed to hash password: {}", e)))?;

            Ok(hash.to_string())
    }

    async fn verify(
        &self,
        password: &Password,
        hash: &str,
    ) -> DomainResult<bool> {
        let argon2 = self.argon2.clone();
        let password = password.clone();
        let hash = hash.to_string();
            let parsed_hash = PasswordHash::new(&hash)
                .map_err(|e| DomainError::internal(format!("Failed to parse hash: {}", e)))?;

            Ok(argon2
                .verify_password(password.as_str().as_bytes(), &parsed_hash)
                .is_ok())
        }
}
