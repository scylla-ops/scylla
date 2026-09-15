use crate::domain::app::AppCredential;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppCredentialId, AppId};
use async_trait::async_trait;

#[async_trait]
pub trait AppCredentialRepository: Send + Sync {
    async fn create(&self, credential: &AppCredential) -> DomainResult<()>;
    async fn find_by_id(&self, id: &AppCredentialId) -> DomainResult<AppCredential>;
    async fn list_by_app(&self, app_id: &AppId) -> DomainResult<Vec<AppCredential>>;
    async fn list_enabled_by_app(&self, app_id: &AppId) -> DomainResult<Vec<AppCredential>>;
    async fn set_enabled(&self, id: &AppCredentialId, enabled: bool) -> DomainResult<()>;
    async fn delete(&self, id: &AppCredentialId) -> DomainResult<()>;
}
