use crate::entities::UserId;
use crate::errors::DomainResult;

/// Port for authentication services
pub trait AuthService: Send + Sync {
    /// Generate an authentication token for a user
    fn generate_token(&self, user_id: &UserId)
    -> impl Future<Output = DomainResult<String>> + Send;

    /// Validate a token
    fn validate_token(&self, token: &str) -> impl Future<Output = DomainResult<bool>> + Send;

    /// Extract user ID from a token
    fn extract_user_id(&self, token: &str) -> impl Future<Output = DomainResult<UserId>> + Send;

    /// Check if a token is expired
    fn is_token_expired(&self, token: &str) -> impl Future<Output = DomainResult<bool>> + Send;

    /// Revoke a token
    fn revoke_token(&self, token: &str) -> impl Future<Output = DomainResult<()>> + Send;
}
