use crate::domain::entities::UserId;
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence for external identity links (`user_oauth_identities`).
#[async_trait]
pub trait OAuthIdentityRepository: Send + Sync {
    async fn find_user_id(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<Option<UserId>>;
    async fn link(
        &self,
        user_id: &UserId,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<()>;
}
