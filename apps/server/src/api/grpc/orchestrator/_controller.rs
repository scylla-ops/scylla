use crate::api::grpc::orchestrator::Orchestrator;
use protocol::services::orchestrator::{
    Ack, Job, StepResult, WorkerHello, WorkerRegistration, orchestrator_server,
};
use protocol::toml;
use protocol::tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use protocol::tonic::codegen::tokio_stream::{Stream, StreamExt};
use protocol::tonic::{Request, Response, Status};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

pub struct OrchestratorController {
    service: Arc<Orchestrator>,
}

impl OrchestratorController {
    pub fn new(service: Arc<Orchestrator>) -> Self { Self { service } }
}

#[async_trait::async_trait]
impl orchestrator_server::Orchestrator for OrchestratorController {
    async fn register_worker(
        &self,
        _request: Request<WorkerHello>,
    ) -> Result<Response<WorkerRegistration>, Status> {
        let new_id = self.service.worker_registry.register().await;
        Ok(Response::new(WorkerRegistration { id: new_id.to_string() }))
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

        let (tx, rx): (Sender<protocol::job::Job>, Receiver<protocol::job::Job>) = mpsc::channel(32);

        self.service
            .worker_registry
            .attach_stream(worker_id, tx.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let base_stream = ReceiverStream::new(rx).map(|job| match toml::to_string(&job.pipeline) {
            Ok(pipeline_toml) => Ok(Job { id: job.id.to_string(), pipeline: pipeline_toml }),
            Err(e) => Err(Status::internal(format!("Failed to serialize pipeline: {}", e))),
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
    }
}
