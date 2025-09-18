use crate::api::grpc::orchestrator::service::OrchestratorService;
use crate::parse_uuid;
use derive_more::Constructor;
use protocol::services::orchestrator::pipeline_event::Event;
use protocol::services::orchestrator::{
    Ack, EventKind, HealthStatus, Job, PipelineEvent, WorkerId, orchestrator_server,
};
use protocol::toml;
use protocol::tonic::codegen::tokio_stream::Stream;
use protocol::tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use protocol::tonic::{Request, Response, Status, Streaming};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use uuid::Uuid;

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
            if let Some(event) = status.event {
                let kind = EventKind::try_from(status.kind).map_err(|_| {
                    Status::invalid_argument(format!("Unknown event kind: {}", status.kind))
                })?;
                let id: Uuid = parse_uuid!(status.id)?;
                match event {
                    Event::Job(_) => self.service.update_job(id, kind.into()).await,
                    Event::Stage(_) => self.service.update_stage(id, kind.into()).await,
                    Event::Step(_) => self.service.update_step(id, kind.into()).await,
                }
            }
        }
        Ok(Response::from(Ack::default()))
    }

    async fn report_health(&self, request: Request<HealthStatus>) -> Result<Response<Ack>, Status> {
        Ok(Response::from(Ack::default()))
    }

    /*async fn register_worker(
        &self,
        _request: Request<WorkerHello>,
    ) -> Result<Response<WorkerRegistration>, Status> {
        let new_id = self.service.worker_registry.register().await;
        Ok(Response::new(WorkerRegistration {
            id: new_id.to_string(),
        }))
    }

    type AssignStream = Pin<Box<dyn Stream<Item = Result<Job, Status>> + Send>>;

    async fn assign(
        &self,
        request: Request<WorkerRegistration>,
    ) -> Result<Response<Self::AssignStream>, Status> {
        let worker_id: Uuid = request
            .into_inner()
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid worker id"))?;

        let (tx, rx): (Sender<protocol::job::Job>, Receiver<protocol::job::Job>) =
            mpsc::channel(32);

        self.service
            .worker_registry
            .attach_stream(worker_id, tx.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let base_stream = ReceiverStream::new(rx).map(|job| match toml::to_string(&job.pipeline) {
            Ok(pipeline_toml) => Ok(Job {
                id: job.id.to_string(),
                pipeline: pipeline_toml,
            }),
            Err(e) => Err(Status::internal(format!(
                "Failed to serialize pipeline: {}",
                e
            ))),
        });

        let reg = self.service.worker_registry.clone();
        tokio::spawn(async move {
            tx.closed().await;
            reg.unregister(worker_id).await;
            tracing::warn!("Worker {} disconnected", worker_id);
        });

        Ok(Response::new(Box::pin(base_stream) as Self::AssignStream))
    }

    async fn report(&self, _request: Request<StepResult>) -> Result<Response<Ack>, Status> {
        // TODO: implement reporting logic in service layer later
        Ok(Response::new(Ack { ok: true }))
    }*/
}

// user -> create a job -> database -> channel to orchestrator
