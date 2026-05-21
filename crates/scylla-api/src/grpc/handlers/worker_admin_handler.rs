use crate::extract_auth_context;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    AppRepository, HashService, PermissionService, PolicyControl, WorkerRepository, WorkerStats,
    WorkerUseCases, WorkerView,
};
use scylla_core::domain::entities::{AppId, OrganizationId};
use scylla_core::domain::value_objects::app::AppName;
use scylla_protocol::services::worker_admin::{
    CreateWorkerRequest, CreatedWorker as ProtoCreatedWorker, DeleteWorkerRequest,
    DeleteWorkerResponse, GetWorkerRequest, GetWorkerStatsRequest, ListWorkersRequest,
    ListWorkersResponse, WorkerStats as ProtoWorkerStats, WorkerView as ProtoWorkerView,
    worker_admin_service_server::WorkerAdminService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// Unary management + introspection of Workers (specialized apps that run jobs).
/// Distinct from the streaming `WorkerService` used by the agent itself.
#[derive(Constructor)]
pub struct WorkerAdminHandler<A, W, H, PC, PS>
where
    A: AppRepository,
    W: WorkerRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
{
    use_cases: Arc<WorkerUseCases<A, W, H, PC, PS>>,
}

#[async_trait::async_trait]
impl<
    A: AppRepository + Send + Sync + 'static,
    W: WorkerRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> WorkerAdminService for WorkerAdminHandler<A, W, H, PC, PS>
{
    async fn create_worker(
        &self,
        request: Request<CreateWorkerRequest>,
    ) -> Result<Response<ProtoCreatedWorker>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let organization_id = OrganizationId::new(&req.organization_id);
        let name = AppName::new(&req.name).map_err(domain_error_to_status)?;

        let created = self
            .use_cases
            .create(&caller, organization_id, name)
            .await
            .map_err(domain_error_to_status)?;

        // A freshly created worker has not connected yet.
        let worker = ProtoWorkerView {
            id: created.app.id().to_string(),
            organization_id: created.app.organization_id().to_string(),
            name: created.app.name().as_str().to_string(),
            is_active: created.app.is_active(),
            created_at: created.app.created_at().to_rfc3339(),
            updated_at: created.app.updated_at().to_rfc3339(),
            connected: false,
            last_seen: String::new(),
        };
        Ok(Response::new(ProtoCreatedWorker {
            worker: Some(worker),
            secret: created.secret.as_str().to_string(),
        }))
    }

    async fn list_workers(
        &self,
        request: Request<ListWorkersRequest>,
    ) -> Result<Response<ListWorkersResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let views = self
            .use_cases
            .list(&caller, OrganizationId::new(&req.organization_id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(ListWorkersResponse {
            workers: views.iter().map(worker_view_to_proto).collect(),
        }))
    }

    async fn get_worker(
        &self,
        request: Request<GetWorkerRequest>,
    ) -> Result<Response<ProtoWorkerView>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let view = self
            .use_cases
            .get(&caller, AppId::new(&req.id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(worker_view_to_proto(&view)))
    }

    async fn get_worker_stats(
        &self,
        request: Request<GetWorkerStatsRequest>,
    ) -> Result<Response<ProtoWorkerStats>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let stats = self
            .use_cases
            .stats(&caller, AppId::new(&req.id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(stats_to_proto(&stats)))
    }

    async fn delete_worker(
        &self,
        request: Request<DeleteWorkerRequest>,
    ) -> Result<Response<DeleteWorkerResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        self.use_cases
            .delete(&caller, AppId::new(&req.id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(DeleteWorkerResponse { deleted: true }))
    }
}

fn worker_view_to_proto(view: &WorkerView) -> ProtoWorkerView {
    ProtoWorkerView {
        id: view.app.id().to_string(),
        organization_id: view.app.organization_id().to_string(),
        name: view.app.name().as_str().to_string(),
        is_active: view.app.is_active(),
        created_at: view.app.created_at().to_rfc3339(),
        updated_at: view.app.updated_at().to_rfc3339(),
        connected: view.connected,
        last_seen: view.last_seen.map(|t| t.to_rfc3339()).unwrap_or_default(),
    }
}

fn stats_to_proto(s: &WorkerStats) -> ProtoWorkerStats {
    ProtoWorkerStats {
        total: s.total,
        pending: s.pending,
        running: s.running,
        completed: s.completed,
        failed: s.failed,
        cancelled: s.cancelled,
        last_run_at: s.last_run_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
    }
}
