use crate::api::grpc::pipeline::repos::surreal::PipelineRepositorySurreal;
use crate::api::grpc::pipeline::snapshot::repos::surreal::PipelineSnapshotRepositorySurreal;
use crate::api::grpc::pipeline::snapshot::service::{
    PipelineSnapshotService, PipelineSnapshotServiceError,
};
use protocol::services::pipeline::snapshot::{
    CreatePipelineSnapshotRequest, CreatePipelineSnapshotResponse, DeletePipelineSnapshotRequest,
    DeletePipelineSnapshotResponse, GetPipelineSnapshotRequest, ListPipelineSnapshotRequest,
    ListPipelineSnapshotResponse, PipelineSnapshotRecord, pipeline_snapshot_server,
};
use protocol::toml;
use protocol::tonic::{Request, Response, Status};

pub struct PipelineSnapshotController;

#[async_trait::async_trait]
impl pipeline_snapshot_server::PipelineSnapshot for PipelineSnapshotController {
    async fn create_pipeline_snapshot(
        &self,
        request: Request<CreatePipelineSnapshotRequest>,
    ) -> Result<Response<CreatePipelineSnapshotResponse>, Status> {
        let CreatePipelineSnapshotRequest { pipeline_id } = request.into_inner();

        let snapshot_id = PipelineSnapshotService::<
            PipelineSnapshotRepositorySurreal,
            PipelineRepositorySurreal,
        >::create_snapshot(pipeline_id)
        .await
        .map_err(map_snapshot_error)?;

        Ok(Response::new(CreatePipelineSnapshotResponse {
            snapshot_id: snapshot_id.to_string(),
        }))
    }

    async fn get_pipeline_snapshot(
        &self,
        request: Request<GetPipelineSnapshotRequest>,
    ) -> Result<Response<PipelineSnapshotRecord>, Status> {
        let GetPipelineSnapshotRequest { snapshot_id } = request.into_inner();

        let record = PipelineSnapshotService::<
            PipelineSnapshotRepositorySurreal,
            PipelineRepositorySurreal,
        >::get_snapshot(snapshot_id)
        .await
        .map_err(map_snapshot_error)?;

        Ok(Response::new(PipelineSnapshotRecord {
            snapshot_id: record.id.key().to_string(),
            pipeline_id: record.pipeline.key().to_string(),
            content: toml::to_string(&record.content)
                .map_err(|e| Status::internal(format!("Failed to serialize pipeline: {}", e)))?,
            created_at: record.created_at.to_string(),
        }))
    }

    async fn delete_pipeline_snapshot(
        &self,
        request: Request<DeletePipelineSnapshotRequest>,
    ) -> Result<Response<DeletePipelineSnapshotResponse>, Status> {
        let DeletePipelineSnapshotRequest { snapshot_id } = request.into_inner();

        PipelineSnapshotService::<PipelineSnapshotRepositorySurreal, PipelineRepositorySurreal>::delete_snapshot(snapshot_id)
            .await
            .map_err(map_snapshot_error)?;

        Ok(Response::new(DeletePipelineSnapshotResponse {}))
    }

    async fn list_pipeline_snapshots(
        &self,
        request: Request<ListPipelineSnapshotRequest>,
    ) -> Result<Response<ListPipelineSnapshotResponse>, Status> {
        let ListPipelineSnapshotRequest { pipeline_id } = request.into_inner();

        let records = PipelineSnapshotService::<
            PipelineSnapshotRepositorySurreal,
            PipelineRepositorySurreal,
        >::list_snapshots(pipeline_id)
        .await
        .map_err(map_snapshot_error)?;

        let response_records = records
            .into_iter()
            .map(|record| PipelineSnapshotRecord {
                snapshot_id: record.id.key().to_string(),
                pipeline_id: record.pipeline.key().to_string(),
                content: toml::to_string(&record.content).unwrap_or_else(|_| String::new()),
                created_at: record.created_at.to_string(),
            })
            .collect();

        Ok(Response::new(ListPipelineSnapshotResponse {
            records: response_records,
        }))
    }
}

fn map_snapshot_error(e: PipelineSnapshotServiceError) -> Status {
    use PipelineSnapshotServiceError as E;
    match e {
        E::Repo(e) => Status::internal(format!("Repository error: {}", e)),
        E::PipelineServiceError(e) => Status::internal(format!("Pipeline service error: {}", e)),
    }
}
