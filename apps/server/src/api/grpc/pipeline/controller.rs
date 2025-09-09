use crate::api::grpc::pipeline::service::{PipelineService, PipelineServiceError};
use crate::parse_uuid;
use derive_more::Constructor;
use protocol::pipeline::Pipeline;
use protocol::services::pipeline::{
    CreatePipelineResponse, DeletePipelineRequest, DeletePipelineResponse, GetPipelineRequest,
    PipelineRecord, PipelineRequest, UpdatePipelineRequest, UpdatePipelineResponse,
    pipeline_server,
};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

#[derive(Constructor)]
pub struct PipelineController {
    service: Arc<PipelineService>,
}

#[async_trait::async_trait]
impl pipeline_server::Pipeline for PipelineController {
    async fn create_pipeline(
        &self,
        request: Request<PipelineRequest>,
    ) -> Result<Response<CreatePipelineResponse>, Status> {
        let PipelineRequest { pipeline_toml } = request.into_inner();
        let id = self
            .service
            .create_pipeline(&pipeline_toml)
            .await
            .map_err(map_err)?;
        Ok(Response::new(CreatePipelineResponse {
            pipeline_id: id.to_string(),
        }))
    }

    async fn get_pipeline(
        &self,
        request: Request<GetPipelineRequest>,
    ) -> Result<Response<PipelineRecord>, Status> {
        let GetPipelineRequest { pipeline_id } = request.into_inner();
        let id = parse_uuid!(pipeline_id)?;
        let rec = self.service.get_pipeline(id).await.map_err(map_err)?;
        // parse TOML to extract name
        let parsed: Pipeline = protocol::toml::from_str(&rec.content)
            .map_err(|_| Status::internal("Failed to parse TOML"))?;
        Ok(Response::new(PipelineRecord {
            pipeline_id: rec.id.to_string(),
            pipeline_toml: rec.content.clone(),
            created_at: rec.created_at.to_string(),
            updated_at: rec.updated_at.to_string(),
            name: Some(parsed.name),
        }))
    }

    async fn delete_pipeline(
        &self,
        request: Request<DeletePipelineRequest>,
    ) -> Result<Response<DeletePipelineResponse>, Status> {
        let DeletePipelineRequest { pipeline_id } = request.into_inner();
        let id = parse_uuid!(pipeline_id)?;
        self.service.delete_pipeline(id).await.map_err(map_err)?;
        Ok(Response::new(DeletePipelineResponse {}))
    }

    async fn update_pipeline(
        &self,
        request: Request<UpdatePipelineRequest>,
    ) -> Result<Response<UpdatePipelineResponse>, Status> {
        let UpdatePipelineRequest {
            pipeline_id,
            pipeline_toml,
        } = request.into_inner();
        let id = parse_uuid!(pipeline_id)?;
        self.service
            .update_pipeline(id, &pipeline_toml)
            .await
            .map_err(map_err)?;
        Ok(Response::new(UpdatePipelineResponse {}))
    }
}

fn map_err(e: PipelineServiceError) -> Status {
    use PipelineServiceError as E;
    match e {
        E::InvalidToml(e) => Status::invalid_argument(format!("Invalid TOML: {}", e)),
        E::Repo(e) => Status::internal(format!("Repository error: {}", e)),
    }
}
