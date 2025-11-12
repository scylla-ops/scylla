use crate::domain::errors::DomainResult;
use crate::domain::value_objects::UserId;
use async_trait::async_trait;

/// Port for authentication services
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Generate an authentication token for a user
    async fn generate_token(&self, user_id: &UserId) -> DomainResult<String>;

    /// Validate a token
    async fn validate_token(&self, token: &str) -> DomainResult<bool>;

    /// Extract user ID from a token
    async fn extract_user_id(&self, token: &str) -> DomainResult<UserId>;

    /// Check if a token is expired
    async fn is_token_expired(&self, token: &str) -> DomainResult<bool>;

    /// Revoke a token
    async fn revoke_token(&self, token: &str) -> DomainResult<()>;
}
