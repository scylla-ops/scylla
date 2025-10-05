use crate::api::grpc::pipeline::service::{PipelineService, PipelineServiceError};
use protocol::services::pipeline::{
    CreatePipelineResponse, DeletePipelineRequest, DeletePipelineResponse, GetPipelineRequest,
    ListPipelinesRequest, ListPipelinesResponse, PipelineRecord, PipelineRequest,
    UpdatePipelineRequest, UpdatePipelineResponse, pipeline_server,
};
use protocol::toml;
use protocol::tonic::{Request, Response, Status};

#[cfg(feature = "surreal")]
use crate::api::grpc::pipeline::repos::surreal::PipelineRepositorySurreal;

#[cfg(feature = "surreal")]
type PipelineRepo = PipelineRepositorySurreal;

pub struct PipelineController;

#[async_trait::async_trait]
impl pipeline_server::Pipeline for PipelineController {
    async fn create_pipeline(
        &self,
        request: Request<PipelineRequest>,
    ) -> Result<Response<CreatePipelineResponse>, Status> {
        let PipelineRequest { pipeline_toml } = request.into_inner();
        let id = PipelineService::<PipelineRepo>::create_pipeline(&pipeline_toml)
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
        let rec = PipelineService::<PipelineRepo>::get_pipeline(pipeline_id)
            .await
            .map_err(map_err)?;

        let pipeline_toml = toml::to_string(&rec.content)
            .map_err(|e| Status::internal(format!("Failed to serialize pipeline: {}", e)))?;

        Ok(Response::new(PipelineRecord {
            pipeline_id: rec.id.to_string(),
            pipeline_toml,
            created_at: rec.created_at.to_string(),
            updated_at: rec.updated_at.to_string(),
            name: Some(rec.content.name),
        }))
    }

    async fn delete_pipeline(
        &self,
        request: Request<DeletePipelineRequest>,
    ) -> Result<Response<DeletePipelineResponse>, Status> {
        let DeletePipelineRequest { pipeline_id } = request.into_inner();
        PipelineService::<PipelineRepo>::delete_pipeline(pipeline_id)
            .await
            .map_err(map_err)?;
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
        PipelineService::<PipelineRepo>::update_pipeline(pipeline_id, &pipeline_toml)
            .await
            .map_err(map_err)?;
        Ok(Response::new(UpdatePipelineResponse {}))
    }

    async fn list_pipelines(
        &self,
        _request: Request<ListPipelinesRequest>,
    ) -> Result<Response<ListPipelinesResponse>, Status> {
        let records = PipelineService::<PipelineRepo>::list_pipelines()
            .await
            .map_err(map_err)?;

        let pipelines = records
            .into_iter()
            .map(|rec| PipelineRecord {
                pipeline_id: rec.id.key().to_string(),
                pipeline_toml: toml::to_string(&rec.content).unwrap_or_default(),
                created_at: rec.created_at.to_string(),
                updated_at: rec.updated_at.to_string(),
                name: Some(rec.content.name),
            })
            .collect();

        Ok(Response::new(ListPipelinesResponse { pipelines }))
    }
}

fn map_err(e: PipelineServiceError) -> Status {
    use PipelineServiceError as E;
    match e {
        E::InvalidToml(e) => Status::invalid_argument(format!("Invalid TOML: {}", e)),
        E::Repo(e) => Status::internal(format!("Repository error: {}", e)),
    }
}
