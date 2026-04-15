use crate::extract_auth_context;
use crate::grpc::mappers::{
    agent_to_proto, domain_error_to_status, domain_to_proto_metadata, proto_to_domain_pagination,
};
use derive_more::Constructor;
use protocol::services::agent::{
    AgentResponse, DeleteAgentRequest, DeleteAgentResponse, GetAgentRequest, ListAgentsRequest,
    ListAgentsResponse, agent_service_server::AgentService,
};
use scylla_core::application::AgentUseCases;
use scylla_core::application::ports::{AgentRepository, PermissionService};
use scylla_core::domain::entities::AgentId;
use scylla_core::domain::value_objects::permission::policy;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct AgentHandler<A: AgentRepository, PS: PermissionService> {
    use_cases: Arc<AgentUseCases<A>>,
    permission_checker: Arc<PS>,
}

#[async_trait::async_trait]
impl<A: AgentRepository + Send + Sync + 'static, PS: PermissionService + Send + Sync + 'static>
    AgentService for AgentHandler<A, PS>
{
    async fn get_agent(
        &self,
        request: Request<GetAgentRequest>,
    ) -> Result<Response<AgentResponse>, Status> {
        let target_id = AgentId::new(&request.get_ref().agent_id);
        require_permission!(self, request, policy::agent::get(target_id));

        let req = request.into_inner();
        let id = AgentId::new(&req.agent_id);
        let agent = self
            .use_cases
            .get(&id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(agent_to_proto(&agent)))
    }

    async fn delete_agent(
        &self,
        request: Request<DeleteAgentRequest>,
    ) -> Result<Response<DeleteAgentResponse>, Status> {
        let target_id = AgentId::new(&request.get_ref().agent_id);
        require_permission!(self, request, policy::agent::delete(target_id));

        let req = request.into_inner();
        let id = AgentId::new(&req.agent_id);
        self.use_cases
            .delete(&id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(DeleteAgentResponse {}))
    }

    async fn list_agents(
        &self,
        request: Request<ListAgentsRequest>,
    ) -> Result<Response<ListAgentsResponse>, Status> {
        require_permission!(self, request, policy::agent::list());

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);
        let result = self
            .use_cases
            .list(pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (agents, metadata) = result.into_parts();
        let agents: Vec<AgentResponse> = agents.iter().map(agent_to_proto).collect();
        Ok(Response::new(ListAgentsResponse {
            agents,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }
}
