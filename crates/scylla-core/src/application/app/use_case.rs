use crate::application::HashService;
use crate::application::app::repository::AppRepository;
use crate::application::caller::CallerContext;
use crate::application::permission::service::PermissionService;
use crate::domain::entities::{App, AppId, OrganizationId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::app::{AppName, AppSecret};
use crate::domain::value_objects::permission::Permission;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

/// What a successful `create` returns: the persisted app plus its plaintext
/// secret, which is presented exactly once and never stored or retrievable.
pub struct CreatedApp {
    pub app: App,
    pub secret: AppSecret,
}

/// Org-scoped management of machine Apps — generic credentials. Every method is
/// Cedar-gated, so an org-admin can manage the apps of orgs they control (admins
/// can manage any). An app is just an identity: it carries no authorization until
/// grants are assigned to it. A *worker* (an app that runs jobs) is provisioned
/// separately via `WorkerUseCases`, which also grants it the worker role.
#[derive(Constructor)]
pub struct AppUseCases<A, H, PS>
where
    A: AppRepository,
    H: HashService,
    PS: PermissionService,
{
    app_repo: Arc<A>,
    hash_service: Arc<H>,
    permission_service: Arc<PS>,
}

impl<A, H, PS> AppUseCases<A, H, PS>
where
    A: AppRepository,
    H: HashService,
    PS: PermissionService,
{
    #[instrument(skip(self, caller), fields(org_id = %organization_id, name = %name))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        organization_id: OrganizationId,
        name: AppName,
    ) -> DomainResult<CreatedApp> {
        self.permission_service
            .check(caller, Permission::CreateApp(organization_id.clone()))
            .await?;

        let secret = AppSecret::generate();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let app = App::create(organization_id, name, secret_hash);

        // A plain app is an identity only — no grant, no policy reload. It gains
        // capabilities later through explicit grants (or becomes a worker).
        self.app_repo.create_app(&app).await?;

        Ok(CreatedApp { app, secret })
    }

    #[instrument(skip(self, caller), fields(org_id = %organization_id))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        organization_id: OrganizationId,
    ) -> DomainResult<Vec<App>> {
        self.permission_service
            .check(
                caller,
                Permission::ListAppsByOrganization(organization_id.clone()),
            )
            .await?;
        self.app_repo.list_by_organization(&organization_id).await
    }

    #[instrument(skip(self, caller), fields(app_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: AppId) -> DomainResult<App> {
        self.permission_service
            .check(caller, Permission::ReadApp(id.clone()))
            .await?;
        self.app_repo.find_by_id(&id).await
    }

    #[instrument(skip(self, caller), fields(app_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: AppId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeleteApp(id.clone()))
            .await?;
        self.app_repo.delete(&id).await
    }
}
