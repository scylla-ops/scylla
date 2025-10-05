use crate::api::grpc::job::repos::surreal::JobRepositorySurreal;
use crate::api::grpc::job::service::{JobCreationResult, JobService, JobServiceError};
use crate::api::grpc::pipeline::repos::surreal::PipelineRepositorySurreal;
use crate::api::grpc::pipeline::snapshot::repos::surreal::PipelineSnapshotRepositorySurreal;
use async_trait::async_trait;
use protocol::services::job::{CreateJobRequest, CreateJobResponse, job_service_server};
use protocol::tonic::{Request, Response, Status};

pub struct JobController;

#[async_trait]
impl job_service_server::JobService for JobController {
    async fn create_job(
        &self,
        request: Request<CreateJobRequest>,
    ) -> Result<Response<CreateJobResponse>, Status> {
        let CreateJobRequest { pipeline_id } = request.into_inner();

        let JobCreationResult {
            job_id,
            snapshot_id,
        } = JobService::<
            JobRepositorySurreal,
            PipelineRepositorySurreal,
            PipelineSnapshotRepositorySurreal,
        >::create_job(pipeline_id)
        .await
        .map_err(map_err)?;

        Ok(Response::new(CreateJobResponse {
            job_id: job_id.to_string(),
            snapshot_id: snapshot_id.to_string(),
        }))
    }
}

fn map_err(e: JobServiceError) -> Status {
    use JobServiceError as E;
    match e {
        E::PipelineService(e) => Status::internal(format!("Pipeline service error: {}", e)),
        E::PipelineSnapshotService(e) => {
            Status::internal(format!("Pipeline snapshot service error: {}", e))
        }
        E::JobRepo(e) => Status::internal(format!("Job repository error: {}", e)),
        E::Pipeline(e) => Status::internal(format!("Pipeline error: {}", e)),
    }
}
