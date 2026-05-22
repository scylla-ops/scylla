use crate::extract_auth_context;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    AppRepository, HashService, PermissionService, PolicyControl, AgentRepository, AgentStats,
    AgentUseCases, AgentView,
};
use scylla_core::domain::entities::{AppId, OrganizationId};
use scylla_core::domain::value_objects::app::AppName;
use scylla_protocol::services::agent_admin::{
    CreateAgentRequest, CreatedAgent as ProtoCreatedAgent, DeleteAgentRequest,
    DeleteAgentResponse, GetAgentRequest, GetAgentStatsRequest, ListAgentsRequest,
    ListAgentsResponse, AgentStats as ProtoAgentStats, AgentView as ProtoAgentView,
    agent_admin_service_server::AgentAdminService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// Unary management + introspection of Agents (specialized apps that run jobs).
/// Distinct from the streaming `AgentService` used by the agent itself.
#[derive(Constructor)]
pub struct AgentAdminHandler<A, W, H, PC, PS>
where
    A: AppRepository,
    W: AgentRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
{
    use_cases: Arc<AgentUseCases<A, W, H, PC, PS>>,
}

#[async_trait::async_trait]
impl<
    A: AppRepository + Send + Sync + 'static,
    W: AgentRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> AgentAdminService for AgentAdminHandler<A, W, H, PC, PS>
{
    async fn create_agent(
        &self,
        request: Request<CreateAgentRequest>,
    ) -> Result<Response<ProtoCreatedAgent>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let organization_id = OrganizationId::new(&req.organization_id);
        let name = AppName::new(&req.name).map_err(domain_error_to_status)?;

        let created = self
            .use_cases
            .create(&caller, organization_id, name)
            .await
            .map_err(domain_error_to_status)?;

        // A freshly created agent has not connected yet.
        let agent = ProtoAgentView {
            id: created.app.id().to_string(),
            organization_id: created.app.organization_id().to_string(),
            name: created.app.name().as_str().to_string(),
            is_active: created.app.is_active(),
            created_at: created.app.created_at().to_rfc3339(),
            updated_at: created.app.updated_at().to_rfc3339(),
            connected: false,
            last_seen: String::new(),
        };
        Ok(Response::new(ProtoCreatedAgent {
            agent: Some(agent),
            secret: created.secret.as_str().to_string(),
        }))
    }

    async fn list_agents(
        &self,
        request: Request<ListAgentsRequest>,
    ) -> Result<Response<ListAgentsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let views = self
            .use_cases
            .list(&caller, OrganizationId::new(&req.organization_id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(ListAgentsResponse {
            agents: views.iter().map(agent_view_to_proto).collect(),
        }))
    }

    async fn get_agent(
        &self,
        request: Request<GetAgentRequest>,
    ) -> Result<Response<ProtoAgentView>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let view = self
            .use_cases
            .get(&caller, AppId::new(&req.id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(agent_view_to_proto(&view)))
    }

    async fn get_agent_stats(
        &self,
        request: Request<GetAgentStatsRequest>,
    ) -> Result<Response<ProtoAgentStats>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let stats = self
            .use_cases
            .stats(&caller, AppId::new(&req.id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(stats_to_proto(&stats)))
    }

    async fn delete_agent(
        &self,
        request: Request<DeleteAgentRequest>,
    ) -> Result<Response<DeleteAgentResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        self.use_cases
            .delete(&caller, AppId::new(&req.id))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(DeleteAgentResponse { deleted: true }))
    }
}

fn agent_view_to_proto(view: &AgentView) -> ProtoAgentView {
    ProtoAgentView {
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

fn stats_to_proto(s: &AgentStats) -> ProtoAgentStats {
    ProtoAgentStats {
        total: s.total,
        pending: s.pending,
        running: s.running,
        completed: s.completed,
        failed: s.failed,
        cancelled: s.cancelled,
        last_run_at: s.last_run_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
    }
}
