use crate::api::grpc::orchestrator::service::OrchestratorService;
use crate::parse_uuid;
use derive_more::Constructor;
use protocol::services::orchestrator::pipeline_event::EventType;
use protocol::services::orchestrator::{
    Ack, EventKind, Job, PipelineEvent, WorkerId, orchestrator_server,
};
use protocol::toml;
use protocol::tonic::codegen::tokio_stream::Stream;
use protocol::tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use protocol::tonic::{Request, Response, Status, Streaming};
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use uuid::Uuid;

static ORCH_TOKEN: OnceLock<String> = OnceLock::new();

#[derive(Constructor)]
pub struct OrchestratorController {
    service: Arc<OrchestratorService>,
}

#[async_trait::async_trait]
impl orchestrator_server::Orchestrator for OrchestratorController {
    type SubscribeJobsStream = Pin<Box<dyn Stream<Item = Result<Job, Status>> + Send>>;

    async fn subscribe_jobs(
        &self,
        request: Request<WorkerId>,
    ) -> Result<Response<Self::SubscribeJobsStream>, Status> {
        let req = request.into_inner();
        let worker_id: Uuid = parse_uuid!(req.id)?;

        let (tx, rx) = mpsc::channel(32);

        let base_stream = ReceiverStream::new(rx).map(|job| match toml::to_string(&job) {
            Ok(job_toml) => Ok(Job { job_toml }),
            Err(e) => Err(Status::internal(format!(
                "Failed to serialize pipeline: {}",
                e
            ))),
        });

        self.service.queue_worker(worker_id, tx).await;

        Ok(Response::new(
            Box::pin(base_stream) as Self::SubscribeJobsStream
        ))
    }

    async fn report_status(
        &self,
        request: Request<Streaming<PipelineEvent>>,
    ) -> Result<Response<Ack>, Status> {
        let mut status_stream = request.into_inner();
        while let Ok(Some(status)) = status_stream.message().await {
            let kind = EventKind::try_from(status.kind).map_err(|_| {
                Status::invalid_argument(format!("Unknown event kind: {}", status.kind))
            })?;
            let event_type = EventType::try_from(status.r#type).map_err(|_| {
                Status::invalid_argument(format!("Unknown event type: {}", status.r#type))
            })?;
            let id: Uuid = parse_uuid!(status.id)?;
            match event_type {
                EventType::Job => self.service.update_job(id, kind.into()).await,
                EventType::Stage => self.service.update_stage(id, kind.into()).await,
                EventType::Step => self.service.update_step(id, kind.into()).await,
            };
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
