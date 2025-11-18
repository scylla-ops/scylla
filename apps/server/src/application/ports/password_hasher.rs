use crate::domain::errors::DomainResult;
use crate::domain::value_objects::Password;
use async_trait::async_trait;

/// Port for password hashing operations
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    /// Hash a plaintext password
    async fn hash(&self, password: &Password) -> DomainResult<String>;

    /// Verify a password against a hash
    async fn verify(&self, password: &Password, hash: &str) -> DomainResult<bool>;
}
