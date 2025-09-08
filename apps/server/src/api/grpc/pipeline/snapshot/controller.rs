use crate::api::grpc::pipeline::snapshot::service::PipelineSnapshotService;
use crate::parse_uuid;
use derive_more::Constructor;
use protocol::services::pipeline::snapshot::{
    CreatePipelineSnapshotRequest, CreatePipelineSnapshotResponse, DeletePipelineSnapshotRequest,
    DeletePipelineSnapshotResponse, GetPipelineSnapshotRequest, ListPipelineSnapshotRequest,
    ListPipelineSnapshotResponse, PipelineSnapshotRecord, pipeline_snapshot_server,
};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

#[derive(Constructor)]
pub struct PipelineSnapshotController {
    service: Arc<PipelineSnapshotService>,
}

#[async_trait::async_trait]
impl pipeline_snapshot_server::PipelineSnapshot for PipelineSnapshotController {
    async fn create_pipeline_snapshot(
        &self,
        request: Request<CreatePipelineSnapshotRequest>,
    ) -> Result<Response<CreatePipelineSnapshotResponse>, Status> {
        let CreatePipelineSnapshotRequest { pipeline_id } = request.into_inner();
        let pipeline_id = parse_uuid!(pipeline_id)?;

        let snapshot_id = self
            .service
            .create_snapshot(pipeline_id)
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
        let snapshot_id = parse_uuid!(snapshot_id)?;

        let record = self
            .service
            .get_snapshot(snapshot_id)
            .await
            .map_err(map_snapshot_error)?;

        Ok(Response::new(PipelineSnapshotRecord {
            snapshot_id: record.id.to_string(),
            pipeline_id: record.pipeline_id.to_string(),
            content: record.content,
            created_at: record.created_at.to_string(),
        }))
    }

    async fn delete_pipeline_snapshot(
        &self,
        request: Request<DeletePipelineSnapshotRequest>,
    ) -> Result<Response<DeletePipelineSnapshotResponse>, Status> {
        let DeletePipelineSnapshotRequest { snapshot_id } = request.into_inner();
        let snapshot_id = parse_uuid!(snapshot_id)?;

        self.service
            .delete_snapshot(snapshot_id)
            .await
            .map_err(map_snapshot_error)?;

        Ok(Response::new(DeletePipelineSnapshotResponse {}))
    }

    async fn list_pipeline_snapshots(
        &self,
        request: Request<ListPipelineSnapshotRequest>,
    ) -> Result<Response<ListPipelineSnapshotResponse>, Status> {
        let ListPipelineSnapshotRequest { pipeline_id } = request.into_inner();
        let pipeline_id = parse_uuid!(pipeline_id)?;

        let records = self
            .service
            .list_snapshots(pipeline_id)
            .await
            .map_err(map_snapshot_error)?;

        let response_records = records
            .into_iter()
            .map(|record| PipelineSnapshotRecord {
                snapshot_id: record.id.to_string(),
                pipeline_id: record.pipeline_id.to_string(),
                content: record.content,
                created_at: record.created_at.to_string(),
            })
            .collect();

        Ok(Response::new(ListPipelineSnapshotResponse {
            records: response_records,
        }))
    }
}

fn map_snapshot_error(
    e: crate::api::grpc::pipeline::snapshot::service::PipelineSnapshotError,
) -> Status {
    use crate::api::grpc::pipeline::snapshot::service::PipelineSnapshotError as E;
    match e {
        E::SendFailed => Status::internal("Critical: unable to send GetPipeline request"),
        E::ReceiveFailed => Status::internal("Critical: unable to receive pipeline"),
        E::PipelineNotFound => Status::not_found("Pipeline not found"),
        E::CreateFailed(_) => Status::internal("Unable to create snapshot"),
        E::GetFailed(_) => Status::internal("Unable to get snapshot"),
        E::DeleteFailed(_) => Status::internal("Unable to delete snapshot"),
        E::ListFailed(_) => Status::internal("Unable to list snapshots"),
    }
}
