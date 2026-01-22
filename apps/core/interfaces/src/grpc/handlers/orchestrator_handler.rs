use crate::grpc::services::services::orchestrator::*;
use derive_more::Constructor;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct OrchestratorHandler {
    container: Arc<AppContainer>,
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

        let response = self.container.run_pipeline_use_case().execute(dto).await?;

        Ok(Response::new(response.into()))
    }
}
