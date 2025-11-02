use crate::application::dto::RunPipelineRequestDto;
use crate::domain::value_objects::PipelineId;
use crate::presentation::grpc::mappers::map_domain_error_to_status;
// use crate::presentation::grpc::middleware::check_permissions;
use crate::shared::di::AppContainer;
use protocol::services::orchestrator::{
    RunPipelineRequest, RunPipelineResponse, orchestrator_server,
};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

pub struct OrchestratorHandler {
    container: Arc<AppContainer>,
}

impl OrchestratorHandler {
    pub fn new(container: Arc<AppContainer>) -> Self {
        Self { container }
    }
}

#[async_trait::async_trait]
impl orchestrator_server::Orchestrator for OrchestratorHandler {
    async fn run_pipeline(
        &self,
        request: Request<RunPipelineRequest>,
    ) -> Result<Response<RunPipelineResponse>, Status> {
        // let pipeline_id = &request.get_ref().pipeline_id;

        // Check RBAC permissions for running/executing pipelines
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     pipeline_id,
        //     "pipelines",
        //     "execute",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = RunPipelineRequestDto {
            pipeline_id: PipelineId::new(req.pipeline_id),
        };

        let response = self
            .container
            .run_pipeline_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(response.into()))
    }
}
