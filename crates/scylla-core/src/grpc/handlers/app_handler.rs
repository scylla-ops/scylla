//! The adapter: every RPC but `revoke_app_secret` and `set_app_secret_enabled` is one `run` and
//! its response. Those two call the use case directly: their permission is on the loaded
//! secret's app, which `Describe` cannot see.

use crate::application::{AppCredentialRepository, AppRepository, AppUseCases, HashService};
use crate::extract_auth_context;
use crate::grpc::adapter::run;
use crate::grpc::convert::id;
use crate::grpc::mappers::{app_credential_to_proto, app_to_proto, domain_error_to_status};
use derive_more::Constructor;
use scylla_auth::authz::{PermissionService, PolicyControl};
use scylla_domain::domain::ids::AppCredentialId;
use scylla_extension::Actions;
use scylla_proto::app::v1::{
    CreateAppRequest, CreateAppResponse, CreateAppSecretRequest, CreateAppSecretResponse,
    DeleteAppRequest, DeleteAppResponse, GetAppRequest, GetAppResponse, ListAppSecretsRequest,
    ListAppSecretsResponse, ListAppsRequest, ListAppsResponse, RevokeAppSecretRequest,
    RevokeAppSecretResponse, SetAppActiveRequest, SetAppActiveResponse, SetAppSecretEnabledRequest,
    SetAppSecretEnabledResponse, app_service_server::AppService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct AppHandler<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService,
    PS: PermissionService,
    PC: PolicyControl,
{
    actions: Arc<Actions>,
    apps: Arc<AppUseCases<A, C, H, PS, PC>>,
}

#[async_trait::async_trait]
impl<
    A: AppRepository + Send + Sync + 'static,
    C: AppCredentialRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
> AppService for AppHandler<A, C, H, PS, PC>
{
    async fn create_app(
        &self,
        request: Request<CreateAppRequest>,
    ) -> Result<Response<CreateAppResponse>, Status> {
        let created = run(&self.actions, &*self.apps, request).await?;
        Ok(Response::new(CreateAppResponse {
            app: Some(app_to_proto(&created.app)),
            secret: created.secret.as_str().to_string(),
        }))
    }

    async fn get_app(
        &self,
        request: Request<GetAppRequest>,
    ) -> Result<Response<GetAppResponse>, Status> {
        let app = run(&self.actions, &*self.apps, request).await?;
        Ok(Response::new(GetAppResponse {
            app: Some(app_to_proto(&app)),
        }))
    }

    async fn list_apps(
        &self,
        request: Request<ListAppsRequest>,
    ) -> Result<Response<ListAppsResponse>, Status> {
        let apps = run(&self.actions, &*self.apps, request).await?;
        Ok(Response::new(ListAppsResponse {
            apps: apps.iter().map(app_to_proto).collect(),
        }))
    }

    async fn delete_app(
        &self,
        request: Request<DeleteAppRequest>,
    ) -> Result<Response<DeleteAppResponse>, Status> {
        run(&self.actions, &*self.apps, request).await?;
        Ok(Response::new(DeleteAppResponse {}))
    }

    async fn set_app_active(
        &self,
        request: Request<SetAppActiveRequest>,
    ) -> Result<Response<SetAppActiveResponse>, Status> {
        let app = run(&self.actions, &*self.apps, request).await?;
        Ok(Response::new(SetAppActiveResponse {
            app: Some(app_to_proto(&app)),
        }))
    }

    async fn create_app_secret(
        &self,
        request: Request<CreateAppSecretRequest>,
    ) -> Result<Response<CreateAppSecretResponse>, Status> {
        let created = run(&self.actions, &*self.apps, request).await?;
        Ok(Response::new(CreateAppSecretResponse {
            app_secret: Some(app_credential_to_proto(&created.credential)),
            secret: created.secret.as_str().to_string(),
        }))
    }

    async fn list_app_secrets(
        &self,
        request: Request<ListAppSecretsRequest>,
    ) -> Result<Response<ListAppSecretsResponse>, Status> {
        let secrets = run(&self.actions, &*self.apps, request).await?;
        Ok(Response::new(ListAppSecretsResponse {
            app_secrets: secrets.iter().map(app_credential_to_proto).collect(),
        }))
    }

    async fn revoke_app_secret(
        &self,
        request: Request<RevokeAppSecretRequest>,
    ) -> Result<Response<RevokeAppSecretResponse>, Status> {
        let caller = caller!(request);
        let secret_id: AppCredentialId = id(request.into_inner().app_secret_id, "app_secret_id")?;
        self.apps
            .revoke_secret(&caller, secret_id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(RevokeAppSecretResponse {}))
    }

    async fn set_app_secret_enabled(
        &self,
        request: Request<SetAppSecretEnabledRequest>,
    ) -> Result<Response<SetAppSecretEnabledResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let secret_id: AppCredentialId = id(req.app_secret_id, "app_secret_id")?;
        let credential = self
            .apps
            .set_secret_enabled(&caller, secret_id, req.enabled)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(SetAppSecretEnabledResponse {
            app_secret: Some(app_credential_to_proto(&credential)),
        }))
    }
}
