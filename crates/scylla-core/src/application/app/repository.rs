use crate::application::permission::grant::Grant;
use crate::domain::entities::{App, AppId, OrganizationId, Worker};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence for machine Apps (`apps` table).
///
/// A plain app is just an identity (`create_app`). A *worker* is a specialized
/// app: `provision_worker` writes the app, its `workers` row and its worker
/// grant atomically, so a worker is never left half-provisioned.
#[async_trait]
pub trait AppRepository: Send + Sync {
    /// Insert a plain app identity (no grant, no worker row).
    async fn create_app(&self, app: &App) -> DomainResult<()>;
    /// Insert an app + its `workers` extension row + its worker grant, atomically.
    async fn provision_worker(
        &self,
        app: &App,
        worker: &Worker,
        grant: &Grant,
    ) -> DomainResult<()>;
    /// Insert an app + a grant, atomically. Retained for the app-token test path.
    async fn provision(&self, app: &App, grant: &Grant) -> DomainResult<()>;
    async fn find_by_id(&self, id: &AppId) -> DomainResult<App>;
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
    ) -> DomainResult<Vec<App>>;
    async fn delete(&self, id: &AppId) -> DomainResult<()>;
}
