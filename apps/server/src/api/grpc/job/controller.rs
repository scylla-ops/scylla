use crate::api::grpc::job::service::{JobCreationResult, JobService, JobServiceError};
use crate::parse_uuid;
use async_trait::async_trait;
use derive_more::Constructor;
use protocol::services::job::{CreateJobRequest, CreateJobResponse, job_service_server};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Constructor)]
pub struct JobController {
    service: Arc<JobService>,
}

#[async_trait]
impl job_service_server::JobService for JobController {
    async fn create_job(
        &self,
        request: Request<CreateJobRequest>,
    ) -> Result<Response<CreateJobResponse>, Status> {
        let CreateJobRequest { pipeline_id } = request.into_inner();
        let uuid: Uuid = parse_uuid!(pipeline_id)?;

        let JobCreationResult {
            job_id,
            snapshot_id,
        } = self.service.create_job(uuid).await.map_err(map_err)?;

        Ok(Response::new(CreateJobResponse {
            job_id: job_id.to_string(),
            snapshot_id: snapshot_id.to_string(),
        }))
    }
}

fn map_err(e: JobServiceError) -> Status {
    use JobServiceError as E;
    match e {
        E::Channel(e) => Status::internal(format!("Critical: unable to use channel {}", e)),
        E::PipelineService(e) => Status::internal(format!("Pipeline service error: {}", e)),
        E::PipelineSnapshotService(e) => {
            Status::internal(format!("Pipeline snapshot service error: {}", e))
        }
        E::ParsePipeline(e) => Status::internal(format!("Unable to parse pipeline: {}", e)),
        E::JobRepo(e) => Status::internal(format!("Job repository error: {}", e)),
    }
}
