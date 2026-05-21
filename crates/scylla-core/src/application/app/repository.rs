use crate::application::permission::grant::Grant;
use crate::domain::entities::{App, AppId, OrganizationId};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence for machine Apps (`apps` table). `provision` writes the app and
/// its initial worker grant atomically, so an app is never left without
/// authorization nor a grant without its app.
#[async_trait]
pub trait AppRepository: Send + Sync {
    async fn provision(&self, app: &App, grant: &Grant) -> DomainResult<()>;
    async fn find_by_id(&self, id: &AppId) -> DomainResult<App>;
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
    ) -> DomainResult<Vec<App>>;
    async fn delete(&self, id: &AppId) -> DomainResult<()>;
}
