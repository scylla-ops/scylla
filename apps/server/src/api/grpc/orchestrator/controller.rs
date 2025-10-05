use crate::api::grpc::orchestrator::service::ORCHESTRATOR_SERVICE;
use derive_more::Constructor;
use protocol::job::{JobData, JobEntry};
use protocol::services::orchestrator::{
    Ack, Job, PipelineStatuUpdate, WorkerId, orchestrator_server,
};
use protocol::toml;
use protocol::tonic::codegen::tokio_stream::Stream;
use protocol::tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use protocol::tonic::{Request, Response, Status, Streaming};
use std::pin::Pin;
use std::sync::OnceLock;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use uuid::Uuid;

macro_rules! parse_uuid {
    ($id:expr) => {
        Uuid::parse_str(&$id).map_err(|e| Status::invalid_argument(format!("Invalid UUID: {}", e)))
    };
}

static ORCH_TOKEN: OnceLock<String> = OnceLock::new();

#[derive(Constructor)]
pub struct OrchestratorController;

#[async_trait::async_trait]
impl orchestrator_server::Orchestrator for OrchestratorController {
    type SubscribeJobStream = Pin<Box<dyn Stream<Item = Result<Job, Status>> + Send>>;

    async fn subscribe_job(
        &self,
        request: Request<WorkerId>,
    ) -> Result<Response<Self::SubscribeJobStream>, Status> {
        let req = request.into_inner();
        let worker_id = parse_uuid!(req.id)?;

        let (tx, rx) = mpsc::channel(32);

        let base_stream =
            ReceiverStream::new(rx).map(|job: JobEntry| match toml::to_string(&job) {
                Ok(job_toml) => Ok(Job {
                    id: job.id,
                    job_toml,
                }),
                Err(e) => Err(Status::internal(format!(
                    "Failed to serialize pipeline: {}",
                    e
                ))),
            });

        ORCHESTRATOR_SERVICE.queue_worker(worker_id, tx).await;

        Ok(Response::new(
            Box::pin(base_stream) as Self::SubscribeJobStream
        ))
    }

    async fn report_status(
        &self,
        request: Request<Streaming<PipelineStatuUpdate>>,
    ) -> Result<Response<Ack>, Status> {
        let mut status_stream = request.into_inner();
        while let Ok(Some(status)) = status_stream.message().await {
            let job_data: JobData = match toml::from_str(&status.job_data_toml) {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("Failed to deserialize job data: {}", e);
                    continue;
                }
            };
            ORCHESTRATOR_SERVICE
                .update_job_data(status.job_id, job_data)
                .await
        }
        Ok(Response::from(Ack::default()))
    }
}

impl OrchestratorController {
    pub fn set_token(token: String) {
        let _ = ORCH_TOKEN.set(token);
    }

    pub fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
        let provided = req
            .metadata()
            .get("x-orch-token")
            .and_then(|v| v.to_str().ok());

        match (ORCH_TOKEN.get(), provided) {
            (Some(expected), Some(v)) if v == expected => Ok(req),
            _ => Err(Status::unauthenticated(
                "Invalid or missing orchestrator token",
            )),
        }
    }
}
