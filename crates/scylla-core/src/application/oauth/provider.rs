use crate::domain::errors::DomainResult;
use crate::domain::user::Email;
use async_trait::async_trait;

pub const PROVIDER_GITHUB: &str = "github";

#[derive(Debug, Clone)]
pub struct OAuthUserInfo {
    pub provider_user_id: String,
    pub email: Option<Email>,
    pub login: String,
}

#[async_trait]
pub trait OAuthProvider: Send + Sync {
    fn authorize_url(&self, state: &str) -> DomainResult<String>;
    async fn exchange_code(&self, code: &str) -> DomainResult<OAuthUserInfo>;
}
