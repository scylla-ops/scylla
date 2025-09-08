use crate::api::grpc::pipeline::snapshot::PipelineSnapshotService;
use crate::api::grpc::pipeline::worker::PipelineMessage;
use crate::parse_uuid;
use protocol::services::pipeline::snapshot::{
    CreatePipelineSnapshotRequest, CreatePipelineSnapshotResponse, DeletePipelineSnapshotRequest,
    DeletePipelineSnapshotResponse, GetPipelineSnapshotRequest, ListPipelineSnapshotRequest,
    ListPipelineSnapshotResponse, PipelineSnapshotRecord, pipeline_snapshot_server,
};
use protocol::tonic::{Request, Response, Status};

#[async_trait::async_trait]
impl pipeline_snapshot_server::PipelineSnapshot for PipelineSnapshotService {
    async fn create_pipeline_snapshot(
        &self,
        request: Request<CreatePipelineSnapshotRequest>,
    ) -> Result<Response<CreatePipelineSnapshotResponse>, Status> {
        let CreatePipelineSnapshotRequest { pipeline_id } = request.into_inner();
        let pipeline_id = parse_uuid!(pipeline_id)?;

        let (tx, rx) = tokio::sync::oneshot::channel();

        self.tx_pipeline
            .send(PipelineMessage::GetPipeline {
                id: pipeline_id,
                respond_tx: tx,
            })
            .await
            .map_err(|_| Status::internal("Critical: unable to send GetPipeline request"))?;

        let pipeline_rec = rx
            .await
            .map_err(|_| Status::internal("Critical: unable to receive pipeline"))?
            .map_err(|e| Status::internal(format!("Failed to retrieve pipeline: {}", e)))?;

        let snapshot_id = self
            .repo
            .create_snapshot(pipeline_rec)
            .await
            .map_err(|e| Status::internal(format!("Unable to create snapshot: {}", e)))?;

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
            .repo
            .get_snapshot(snapshot_id)
            .await
            .map_err(|e| Status::internal(format!("Unable to get snapshot: {}", e)))?;

        Ok(Response::new(PipelineSnapshotRecord {
            snapshot_id: snapshot_id.to_string(),
            pipeline_id: record.pipeline_id.to_string(),
            content: record.content,
            created_at: record.created_at.to_string(),
        }))
    }

    async fn delete_pipeline_snapshot(
        &self,
        _request: Request<DeletePipelineSnapshotRequest>,
    ) -> Result<Response<DeletePipelineSnapshotResponse>, Status> {
        //todo soft delete ?
        todo!()
    }

    async fn list_pipeline_snapshots(
        &self,
        request: Request<ListPipelineSnapshotRequest>,
    ) -> Result<Response<ListPipelineSnapshotResponse>, Status> {
        let ListPipelineSnapshotRequest { pipeline_id } = request.into_inner();
        let pipeline_id = parse_uuid!(pipeline_id)?;

        let (tx, rx) = tokio::sync::oneshot::channel();

        self.tx_pipeline
            .send(PipelineMessage::GetPipeline {
                id: pipeline_id,
                respond_tx: tx,
            })
            .await
            .map_err(|_| Status::internal("Critical: unable to send GetPipeline request"))?;

        let pipeline_rec = rx
            .await
            .map_err(|_| Status::internal("Critical: unable to receive pipeline"))?
            .map_err(|e| Status::internal(format!("Failed to retrieve pipeline: {}", e)))?;

        let records = self
            .repo
            .list_snapshots(pipeline_rec)
            .await
            .map_err(|e| Status::internal(format!("Unable to list snapshots: {}", e)))?;

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
