use crate::domain::app::AppCredential;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppCredentialId, AppId};
use async_trait::async_trait;

/// Persistence for App secrets (`app_secrets` table). An App can hold several;
/// authentication accepts the plaintext of any *enabled* one. Revoking removes
/// the row; disabling keeps it but flips `enabled` off.
#[async_trait]
pub trait AppCredentialRepository: Send + Sync {
    /// Insert a new secret for an existing app.
    async fn create(&self, credential: &AppCredential) -> DomainResult<()>;
    async fn find_by_id(&self, id: &AppCredentialId) -> DomainResult<AppCredential>;
    /// All secrets of an app (enabled + disabled), newest first.
    async fn list_by_app(&self, app_id: &AppId) -> DomainResult<Vec<AppCredential>>;
    /// Only the enabled secrets of an app — used by the token exchange.
    async fn list_enabled_by_app(&self, app_id: &AppId) -> DomainResult<Vec<AppCredential>>;
    /// Enable / disable a secret without deleting it.
    async fn set_enabled(&self, id: &AppCredentialId, enabled: bool) -> DomainResult<()>;
    /// Permanently remove a secret (revoke).
    async fn delete(&self, id: &AppCredentialId) -> DomainResult<()>;
}
