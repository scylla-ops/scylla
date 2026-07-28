use crate::domain::entities::AppToken;
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence for App bearer tokens (`app_tokens` table). Separate from user
/// sessions; read by the auth interceptor to resolve a token to an App.
#[async_trait]
pub trait AppTokenRepository: Send + Sync {
    async fn create(&self, token: &AppToken) -> DomainResult<()>;
    async fn find_by_token(&self, token: &str) -> DomainResult<AppToken>;
}
