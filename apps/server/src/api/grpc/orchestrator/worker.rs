use crate::api::grpc::BackgroundWorker;
use crate::api::grpc::orchestrator::service::{OrchestratorService, WorkerRecord};
use derive_more::Constructor;
use protocol::job::Job;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::warn;

#[derive(Constructor)]
pub struct OchestratorWorker {
    service: Arc<OrchestratorService>,
    rx_job: mpsc::Receiver<OrchestratorMessage>,
}

#[derive(Debug)]
pub enum OrchestratorMessage {
    NewJob { job: Job },
}

impl BackgroundWorker for OchestratorWorker {
    fn spawn_worker(mut self, mut shutdown: tokio::sync::watch::Receiver<bool>) -> JoinHandle<()> {
        tokio::spawn(async move {
            'main: loop {
                tokio::select! {
                    _ = shutdown.changed() => {
                        if *shutdown.borrow() {
                            self.service.shutdown().await;
                            warn!("Orchestrator worker shutdown");
                            break 'main;
                        }
                    }
                    Some(mes) = self.rx_job.recv() => {
                        self.handle_message(mes).await;
                    }
                }
            }
        })
    }
}

impl OchestratorWorker {
    async fn handle_message(&self, message: OrchestratorMessage) {
        match message {
            OrchestratorMessage::NewJob { job } => {
                if let Ok(WorkerRecord { tx_job, .. }) = self.service.get_first_available().await {
                    let _ = tx_job.send(job).await;
                } else {
                    warn!("No worker available to handle job")
                }
            }
        }
    }
}
