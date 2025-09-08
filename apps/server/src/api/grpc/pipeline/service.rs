use crate::api::grpc::pipeline::PipelineService;
use protocol::pipeline::Pipeline;
use protocol::services::pipeline::{
    CreatePipelineResponse, DeletePipelineRequest, DeletePipelineResponse, GetPipelineRequest,
    PipelineRecord, PipelineRequest, UpdatePipelineRequest, UpdatePipelineResponse,
    pipeline_server,
};
use protocol::toml;
use protocol::tonic::{Request, Response, Status};
use uuid::Uuid;

#[async_trait::async_trait]
impl pipeline_server::Pipeline for PipelineService {
    async fn create_pipeline(
        &self,
        request: Request<PipelineRequest>,
    ) -> Result<Response<CreatePipelineResponse>, Status> {
        let parsed_pipeline: Pipeline = toml::from_str(&request.into_inner().pipeline_toml)
            .map_err(|e| Status::invalid_argument(format!("Invalid pipeline TOML: {}", e)))?;

        let id = self
            .repo
            .create_pipeline(parsed_pipeline)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreatePipelineResponse {
            pipeline_id: id.to_string(),
        }))
    }

    async fn get_pipeline(
        &self,
        request: Request<GetPipelineRequest>,
    ) -> Result<Response<PipelineRecord>, Status> {
        let pipeline_uuid = Uuid::parse_str(&request.into_inner().pipeline_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid pipeline ID: {}", e)))?;

        let record = self
            .repo
            .get_pipeline(pipeline_uuid)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let parsed_toml: Pipeline = toml::from_str(&record.content)
            .map_err(|e| Status::internal(format!("Failed to parse TOML: {}", e)))?;

        Ok(Response::new(PipelineRecord {
            pipeline_id: String::from(record.id),
            pipeline_toml: record.content.clone(),
            created_at: record.created_at.to_string(),
            updated_at: record.updated_at.to_string(),
            name: Option::from(parsed_toml.name),
        }))
    }

    async fn delete_pipeline(
        &self,
        request: Request<DeletePipelineRequest>,
    ) -> Result<Response<DeletePipelineResponse>, Status> {
        let pipeline_uuid = Uuid::parse_str(&request.into_inner().pipeline_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid pipeline ID: {}", e)))?;

        self.repo
            .delete_pipeline(pipeline_uuid)
            .await
            .map_err(|e| Status::not_found(format!("{}", e)))?;

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
        let pipeline_uuid = Uuid::parse_str(&pipeline_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid pipeline ID: {}", e)))?;
        let parsed_pipeline: Pipeline = toml::from_str(&pipeline_toml)
            .map_err(|e| Status::invalid_argument(format!("Invalid pipeline TOML: {}", e)))?;

        self.repo
            .update_pipeline(pipeline_uuid, parsed_pipeline)
            .await
            .map_err(|e| Status::not_found(format!("{}", e)))?;

        Ok(Response::new(UpdatePipelineResponse {}))
    }
}
