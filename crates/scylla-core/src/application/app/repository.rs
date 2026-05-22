use crate::application::permission::grant::Grant;
use crate::domain::entities::{App, AppCredential, AppId, OrganizationId, Agent};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence for machine Apps (`apps` table).
///
/// A plain app is just an identity (`create_app`). A *agent* is a specialized
/// app: `provision_agent` writes the app, its `agents` row and its agent
/// grant atomically, so a agent is never left half-provisioned. Every creation
/// path also writes the App's initial secret (`app_secrets`) in the same tx, so
/// an App is never left without a usable credential.
#[async_trait]
pub trait AppRepository: Send + Sync {
    /// Insert a plain app identity + its initial secret (no grant, no agent row).
    async fn create_app(&self, app: &App, credential: &AppCredential) -> DomainResult<()>;
    /// Insert an app + its initial secret + `agents` row + agent grant, atomically.
    async fn provision_agent(
        &self,
        app: &App,
        credential: &AppCredential,
        agent: &Agent,
        grant: &Grant,
    ) -> DomainResult<()>;
    /// Insert an app + its initial secret + a grant, atomically. Test path.
    async fn provision(
        &self,
        app: &App,
        credential: &AppCredential,
        grant: &Grant,
    ) -> DomainResult<()>;
    async fn find_by_id(&self, id: &AppId) -> DomainResult<App>;
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
    ) -> DomainResult<Vec<App>>;
    /// Enable / disable the app (gates token issuance + resolution).
    async fn set_active(&self, id: &AppId, active: bool) -> DomainResult<()>;
    async fn delete(&self, id: &AppId) -> DomainResult<()>;
}
