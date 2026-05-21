use crate::extract_auth_context;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    AppRepository, AppUseCases, HashService, PermissionService, PolicyControl, WorkerDispatch,
};
use scylla_core::domain::entities::{App, AppId, OrganizationId};
use scylla_core::domain::value_objects::app::AppName;
use scylla_core::infrastructure::InMemoryWorkerRegistry;
use scylla_protocol::services::app::{
    App as ProtoApp, CreateAppRequest, CreatedApp as ProtoCreatedApp, DeleteAppRequest,
    DeleteAppResponse, GetAppRequest, ListAppsRequest, ListAppsResponse,
    app_service_server::AppService,
};
use std::collections::HashSet;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct AppHandler<A, H, PC, PS>
where
    A: AppRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
{
    use_cases: Arc<AppUseCases<A, H, PC, PS>>,
    /// Used to report which apps currently have an open worker stream (online).
    registry: Arc<InMemoryWorkerRegistry>,
}

impl<A, H, PC, PS> AppHandler<A, H, PC, PS>
where
    A: AppRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
{
    fn connected_ids(&self) -> HashSet<String> {
        self.registry
            .connected()
            .iter()
            .map(|id| id.as_str().to_string())
            .collect()
    }
}

#[async_trait::async_trait]
impl<
    A: AppRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> AppService for AppHandler<A, H, PC, PS>
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

        // A freshly created app has not connected yet.
        Ok(Response::new(ProtoCreatedApp {
            app: Some(app_to_proto(&created.app, false)),
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
        let connected = self.connected_ids().contains(app.id().as_str());
        Ok(Response::new(app_to_proto(&app, connected)))
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
        let connected = self.connected_ids();
        Ok(Response::new(ListAppsResponse {
            apps: apps
                .iter()
                .map(|a| app_to_proto(a, connected.contains(a.id().as_str())))
                .collect(),
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
}

fn app_to_proto(a: &App, connected: bool) -> ProtoApp {
    ProtoApp {
        id: a.id().to_string(),
        organization_id: a.organization_id().to_string(),
        name: a.name().as_str().to_string(),
        is_active: a.is_active(),
        created_at: a.created_at().to_rfc3339(),
        updated_at: a.updated_at().to_rfc3339(),
        connected,
    }
}
