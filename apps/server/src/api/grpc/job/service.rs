use crate::api::grpc::job::JobService;
use async_trait::async_trait;
use protocol::services::job::{CreateJobRequest, CreateJobResponse, job_service_server};
use protocol::tonic::{Request, Response, Status};

#[async_trait]
impl job_service_server::JobService for JobService {
    async fn create_job(
        &self,
        _request: Request<CreateJobRequest>,
    ) -> Result<Response<CreateJobResponse>, Status> {
        todo!()
    }
}
