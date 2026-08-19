use crate::application::{
    AgentRepository, AgentStats, AgentUseCases, AgentView, AppRepository, HashService,
    PermissionService, PolicyControl,
};
use crate::extract_auth_context;
use crate::grpc::convert::{required, ts, wrap};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::domain::app::AppName;
use scylla_core::domain::ids::{AppId, OrganizationId};
use scylla_protocol::agent::v1::{
    Agent as ProtoAgent, AgentStats as ProtoAgentStats, CreateAgentRequest, CreateAgentResponse,
    DailyOutcome as ProtoDailyOutcome, DeleteAgentRequest, DeleteAgentResponse, GetAgentRequest,
    GetAgentResponse, GetAgentStatsRequest, GetAgentStatsResponse, ListAgentsRequest,
    ListAgentsResponse, agent_admin_service_server::AgentAdminService,
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
    ) -> Result<Response<CreateAgentResponse>, Status> {
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

        // A freshly created agent has not connected yet, so it holds no jobs.
        let agent = ProtoAgent {
            agent_id: wrap(created.app.id().to_string()),
            organization_id: wrap(created.app.organization_id().to_string()),
            name: created.app.name().as_str().to_string(),
            is_active: created.app.is_active(),
            created_at: ts(created.app.created_at()),
            updated_at: ts(created.app.updated_at()),
            connected: false,
            last_seen: None,
            in_flight: 0,
        };
        Ok(Response::new(CreateAgentResponse {
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
            .list(
                &caller,
                OrganizationId::new(&required(req.organization_id, "organization_id")?),
            )
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(ListAgentsResponse {
            agents: views.iter().map(agent_view_to_proto).collect(),
        }))
    }

    async fn get_agent(
        &self,
        request: Request<GetAgentRequest>,
    ) -> Result<Response<GetAgentResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let view = self
            .use_cases
            .get(&caller, AppId::new(&required(req.agent_id, "agent_id")?))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(GetAgentResponse {
            agent: Some(agent_view_to_proto(&view)),
        }))
    }

    async fn get_agent_stats(
        &self,
        request: Request<GetAgentStatsRequest>,
    ) -> Result<Response<GetAgentStatsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let stats = self
            .use_cases
            .stats(&caller, AppId::new(&required(req.agent_id, "agent_id")?))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(GetAgentStatsResponse {
            stats: Some(stats_to_proto(&stats)),
        }))
    }

    async fn delete_agent(
        &self,
        request: Request<DeleteAgentRequest>,
    ) -> Result<Response<DeleteAgentResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        self.use_cases
            .delete(&caller, AppId::new(&required(req.agent_id, "agent_id")?))
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(DeleteAgentResponse {}))
    }
}

fn agent_view_to_proto(view: &AgentView) -> ProtoAgent {
    ProtoAgent {
        agent_id: wrap(view.app.id().to_string()),
        organization_id: wrap(view.app.organization_id().to_string()),
        name: view.app.name().as_str().to_string(),
        is_active: view.app.is_active(),
        created_at: ts(view.app.created_at()),
        updated_at: ts(view.app.updated_at()),
        connected: view.connected,
        last_seen: view.last_seen.and_then(ts),
        // Bounded by the registry's dispatch queue, so it always fits — a
        // saturating cast keeps an absurd count from wrapping negative.
        in_flight: i32::try_from(view.in_flight).unwrap_or(i32::MAX),
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
        orphaned: s.orphaned,
        last_run_at: s.last_run_at.and_then(ts),
        median_duration_ms: s.median_duration_ms,
        p95_duration_ms: s.p95_duration_ms,
        daily: s
            .daily
            .iter()
            .map(|d| ProtoDailyOutcome {
                day: ts(d.day),
                completed: d.completed,
                failed: d.failed,
                cancelled: d.cancelled,
                orphaned: d.orphaned,
                median_duration_ms: d.median_duration_ms,
            })
            .collect(),
    }
}
