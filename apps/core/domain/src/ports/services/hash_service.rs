use crate::errors::DomainResult;
use crate::value_objects::user::Password;
use async_trait::async_trait;

/// Port for hashing services
#[async_trait]
pub trait HashService {
    /// Hash a plaintext password
    async fn hash(&self, password: &Password) -> DomainResult<String>;

    /// Verify a password against a hash
    async fn verify(&self, password: &Password, hash: &str) -> DomainResult<bool>;
}
