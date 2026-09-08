use crate::application::{
    AppCredentialRepository, AppRepository, AppUseCases, HashService, PermissionService,
    PolicyControl,
};
use crate::extract_auth_context;
use crate::grpc::convert::{required, ts, wrap};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::domain::app::{App, AppCredential};
use scylla_core::domain::app::{AppName, AppSecretLabel};
use scylla_core::domain::ids::{AppCredentialId, AppId, OrganizationId};
use scylla_protocol::app::v1::{
    App as ProtoApp, AppSecret as ProtoAppSecret, CreateAppRequest, CreateAppResponse,
    CreateAppSecretRequest, CreateAppSecretResponse, DeleteAppRequest, DeleteAppResponse,
    GetAppRequest, GetAppResponse, ListAppSecretsRequest, ListAppSecretsResponse, ListAppsRequest,
    ListAppsResponse, RevokeAppSecretRequest, RevokeAppSecretResponse, SetAppActiveRequest,
    SetAppActiveResponse, SetAppSecretEnabledRequest, SetAppSecretEnabledResponse,
    app_service_server::AppService,
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
    use_cases: Arc<AppUseCases<A, C, H, PS, PC>>,
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
        let caller = caller!(request);
        let req = request.into_inner();
        let organization_id =
            OrganizationId::new(&required(req.organization_id, "organization_id")?);
        let name = AppName::new(&req.name).map_err(domain_error_to_status)?;

        let created = self
            .use_cases
            .create(&caller, organization_id, name)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(CreateAppResponse {
            app: Some(app_to_proto(&created.app)),
            secret: created.secret.as_str().to_string(),
        }))
    }

    async fn get_app(
        &self,
        request: Request<GetAppRequest>,
    ) -> Result<Response<GetAppResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let app = self
            .use_cases
            .get(&caller, AppId::new(&required(req.app_id, "app_id")?))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(GetAppResponse {
            app: Some(app_to_proto(&app)),
        }))
    }

    async fn list_apps(
        &self,
        request: Request<ListAppsRequest>,
    ) -> Result<Response<ListAppsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let apps = self
            .use_cases
            .list(
                &caller,
                OrganizationId::new(&required(req.organization_id, "organization_id")?),
            )
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(ListAppsResponse {
            apps: apps.iter().map(app_to_proto).collect(),
        }))
    }

    async fn delete_app(
        &self,
        request: Request<DeleteAppRequest>,
    ) -> Result<Response<DeleteAppResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        self.use_cases
            .delete(&caller, AppId::new(&required(req.app_id, "app_id")?))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(DeleteAppResponse {}))
    }

    async fn set_app_active(
        &self,
        request: Request<SetAppActiveRequest>,
    ) -> Result<Response<SetAppActiveResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let app = self
            .use_cases
            .set_active(
                &caller,
                AppId::new(&required(req.app_id, "app_id")?),
                req.is_active,
            )
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(SetAppActiveResponse {
            app: Some(app_to_proto(&app)),
        }))
    }

    async fn create_app_secret(
        &self,
        request: Request<CreateAppSecretRequest>,
    ) -> Result<Response<CreateAppSecretResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let app_id = AppId::new(&required(req.app_id, "app_id")?);
        let label = AppSecretLabel::new(&req.label).map_err(domain_error_to_status)?;

        let created = self
            .use_cases
            .create_secret(&caller, app_id, label)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(CreateAppSecretResponse {
            app_secret: Some(credential_to_proto(&created.credential)),
            secret: created.secret.as_str().to_string(),
        }))
    }

    async fn list_app_secrets(
        &self,
        request: Request<ListAppSecretsRequest>,
    ) -> Result<Response<ListAppSecretsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let secrets = self
            .use_cases
            .list_secrets(&caller, AppId::new(&required(req.app_id, "app_id")?))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(ListAppSecretsResponse {
            app_secrets: secrets.iter().map(credential_to_proto).collect(),
        }))
    }

    async fn revoke_app_secret(
        &self,
        request: Request<RevokeAppSecretRequest>,
    ) -> Result<Response<RevokeAppSecretResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        self.use_cases
            .revoke_secret(
                &caller,
                AppCredentialId::new(&required(req.app_secret_id, "app_secret_id")?),
            )
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
        let credential = self
            .use_cases
            .set_secret_enabled(
                &caller,
                AppCredentialId::new(&required(req.app_secret_id, "app_secret_id")?),
                req.enabled,
            )
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(SetAppSecretEnabledResponse {
            app_secret: Some(credential_to_proto(&credential)),
        }))
    }
}

fn app_to_proto(a: &App) -> ProtoApp {
    ProtoApp {
        app_id: wrap(a.id().to_string()),
        organization_id: wrap(a.organization_id().to_string()),
        name: a.name().as_str().to_string(),
        is_active: a.is_active(),
        created_at: ts(a.created_at()),
        updated_at: ts(a.updated_at()),
    }
}

fn credential_to_proto(c: &AppCredential) -> ProtoAppSecret {
    ProtoAppSecret {
        app_secret_id: wrap(c.id().to_string()),
        app_id: wrap(c.app_id().to_string()),
        label: c.label().as_str().to_string(),
        enabled: c.is_enabled(),
        created_at: ts(c.created_at()),
        updated_at: ts(c.updated_at()),
    }
}
