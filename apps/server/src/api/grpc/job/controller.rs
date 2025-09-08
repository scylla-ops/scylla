use crate::api::grpc::job::JobService;
use async_trait::async_trait;
use protocol::services::job::{CreateJobRequest, CreateJobResponse, job_service_server};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

pub struct JobController {
    service: Arc<JobService>,
}

impl JobController {
    pub fn new(service: Arc<JobService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl job_service_server::JobService for JobController {
    async fn create_job(
        &self,
        _request: Request<CreateJobRequest>,
    ) -> Result<Response<CreateJobResponse>, Status> {
        // TODO: delegate to service when implemented
        Err(Status::unimplemented("create_job not implemented yet"))
    }
}
