use crate::extract_auth_context;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    AppCredentialRepository, AppRepository, AppUseCases, HashService, PermissionService,
};
use scylla_core::domain::entities::{App, AppCredential, AppCredentialId, AppId, OrganizationId};
use scylla_core::domain::value_objects::app::{AppName, AppSecretLabel};
use scylla_protocol::services::app::{
    App as ProtoApp, AppSecret as ProtoAppSecret, CreateAppRequest, CreateAppSecretRequest,
    CreatedApp as ProtoCreatedApp, CreatedAppSecret as ProtoCreatedAppSecret, DeleteAppRequest,
    DeleteAppResponse, GetAppRequest, ListAppSecretsRequest, ListAppSecretsResponse,
    ListAppsRequest, ListAppsResponse, RevokeAppSecretRequest, RevokeAppSecretResponse,
    SetAppActiveRequest, SetAppSecretEnabledRequest, app_service_server::AppService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct AppHandler<A, C, H, PS>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService,
    PS: PermissionService,
{
    use_cases: Arc<AppUseCases<A, C, H, PS>>,
}

#[async_trait::async_trait]
impl<
    A: AppRepository + Send + Sync + 'static,
    C: AppCredentialRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> AppService for AppHandler<A, C, H, PS>
{
    async fn create_app(
        &self,
        request: Request<CreateAppRequest>,
    ) -> Result<Response<ProtoCreatedApp>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let organization_id = OrganizationId::new(&req.organization_id);
        let name = AppName::new(&req.name).map_err(domain_error_to_status)?;

        let created = self
            .use_cases
            .create(&caller, organization_id, name)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ProtoCreatedApp {
            app: Some(app_to_proto(&created.app)),
            secret: created.secret.as_str().to_string(),
        }))
    }

    async fn get_app(&self, request: Request<GetAppRequest>) -> Result<Response<ProtoApp>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let app = self
            .use_cases
            .get(&caller, AppId::new(&req.id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(app_to_proto(&app)))
    }

    async fn list_apps(
        &self,
        request: Request<ListAppsRequest>,
    ) -> Result<Response<ListAppsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let apps = self
            .use_cases
            .list(&caller, OrganizationId::new(&req.organization_id))
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
            .delete(&caller, AppId::new(&req.id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(DeleteAppResponse { deleted: true }))
    }

    async fn set_app_active(
        &self,
        request: Request<SetAppActiveRequest>,
    ) -> Result<Response<ProtoApp>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let app = self
            .use_cases
            .set_active(&caller, AppId::new(&req.app_id), req.active)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(app_to_proto(&app)))
    }

    async fn create_app_secret(
        &self,
        request: Request<CreateAppSecretRequest>,
    ) -> Result<Response<ProtoCreatedAppSecret>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let app_id = AppId::new(&req.app_id);
        let label = AppSecretLabel::new(&req.label).map_err(domain_error_to_status)?;

        let created = self
            .use_cases
            .create_secret(&caller, app_id, label)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ProtoCreatedAppSecret {
            credential: Some(credential_to_proto(&created.credential)),
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
            .list_secrets(&caller, AppId::new(&req.app_id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(ListAppSecretsResponse {
            secrets: secrets.iter().map(credential_to_proto).collect(),
        }))
    }

    async fn revoke_app_secret(
        &self,
        request: Request<RevokeAppSecretRequest>,
    ) -> Result<Response<RevokeAppSecretResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        self.use_cases
            .revoke_secret(&caller, AppCredentialId::new(&req.secret_id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(RevokeAppSecretResponse { deleted: true }))
    }

    async fn set_app_secret_enabled(
        &self,
        request: Request<SetAppSecretEnabledRequest>,
    ) -> Result<Response<ProtoAppSecret>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let credential = self
            .use_cases
            .set_secret_enabled(&caller, AppCredentialId::new(&req.secret_id), req.enabled)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(credential_to_proto(&credential)))
    }
}

fn app_to_proto(a: &App) -> ProtoApp {
    ProtoApp {
        id: a.id().to_string(),
        organization_id: a.organization_id().to_string(),
        name: a.name().as_str().to_string(),
        is_active: a.is_active(),
        created_at: a.created_at().to_rfc3339(),
        updated_at: a.updated_at().to_rfc3339(),
    }
}

fn credential_to_proto(c: &AppCredential) -> ProtoAppSecret {
    ProtoAppSecret {
        id: c.id().to_string(),
        app_id: c.app_id().to_string(),
        label: c.label().as_str().to_string(),
        enabled: c.is_enabled(),
        created_at: c.created_at().to_rfc3339(),
        updated_at: c.updated_at().to_rfc3339(),
    }
}
