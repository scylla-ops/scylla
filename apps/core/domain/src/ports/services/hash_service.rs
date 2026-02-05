use crate::errors::DomainResult;
use crate::value_objects::user::Password;

/// Port for hashing services
pub trait HashService: Send + Sync {
    /// Hash a plaintext password
    fn hash(&self, password: &Password) -> impl Future<Output = DomainResult<String>> + Send;

    /// Verify a password against a hash
    fn verify(
        &self,
        password: &Password,
        hash: &str,
    ) -> impl Future<Output = DomainResult<bool>> + Send;
}
