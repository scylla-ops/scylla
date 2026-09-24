//! The app's reads. One block per query, in the order it runs: the struct, its permission, its
//! output type, what `Fetch` reads.

use super::AppUseCases;
use crate::application::{AppCredentialRepository, AppRepository, HashService};
use crate::domain::app::{App, AppCredential};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, OrganizationId};
use crate::domain::permission::Permission;
use async_trait::async_trait;
use scylla_auth::authz::{PermissionService, PolicyControl};
use scylla_extension::{Authorized, Describe, Fetch, Fetched, Query, Run};

#[derive(Debug)]
pub struct GetApp {
    pub id: AppId,
}

impl Describe for GetApp {
    fn permission(&self) -> Permission {
        Permission::ReadApp(self.id.clone())
    }
}

impl Query for GetApp {
    type Output = App;
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Fetch<GetApp>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<GetApp>) -> DomainResult<Fetched<GetApp>> {
        let app = self.app_repo.find_by_id(&input.command().id).await?;
        Ok(input.fetched(app))
    }
}

#[derive(Debug)]
pub struct ListApps {
    pub organization_id: OrganizationId,
}

impl Describe for ListApps {
    fn permission(&self) -> Permission {
        Permission::ListAppsByOrganization(self.organization_id.clone())
    }
}

impl Query for ListApps {
    type Output = Vec<App>;
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Fetch<ListApps>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<ListApps>) -> DomainResult<Fetched<ListApps>> {
        let apps = self
            .app_repo
            .list_by_organization(&input.command().organization_id)
            .await?;
        Ok(input.fetched(apps))
    }
}

#[derive(Debug)]
pub struct ListAppSecrets {
    pub app_id: AppId,
}

impl Describe for ListAppSecrets {
    fn permission(&self) -> Permission {
        Permission::ReadApp(self.app_id.clone())
    }
}

impl Query for ListAppSecrets {
    type Output = Vec<AppCredential>;
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Fetch<ListAppSecrets>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(
        &self,
        input: Authorized<ListAppSecrets>,
    ) -> DomainResult<Fetched<ListAppSecrets>> {
        let secrets = self
            .credential_repo
            .list_by_app(&input.command().app_id)
            .await?;
        Ok(input.fetched(secrets))
    }
}
