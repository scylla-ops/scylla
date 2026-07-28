use crate::domain::errors::DomainResult;
use crate::domain::user::Email;
use async_trait::async_trait;

pub const PROVIDER_GITHUB: &str = "github";

/// Normalised identity returned by an OAuth provider after a successful code
/// exchange.
#[derive(Debug, Clone)]
pub struct OAuthUserInfo {
    pub provider_user_id: String,
    pub email: Option<Email>,
    pub login: String,
}

/// OAuth provider port (e.g. GitHub). The concrete HTTP implementation lives in
/// the infrastructure layer behind the `oauth-github` feature; tests stub it.
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    fn authorize_url(&self, state: &str) -> DomainResult<String>;
    async fn exchange_code(&self, code: &str) -> DomainResult<OAuthUserInfo>;
}
