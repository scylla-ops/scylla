use crate::api::grpc::BackgroundWorker;
use crate::api::grpc::pipeline::PipelineRepository;
use crate::api::grpc::pipeline::models::PipelineRecord;
use derive_more::Constructor;
use std::sync::Arc;
use tokio::sync::watch::Receiver;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Debug)]
pub enum PipelineMessage {
    GetPipeline {
        id: Uuid,
        respond_tx: oneshot::Sender<anyhow::Result<PipelineRecord>>,
    },
}

#[derive(Constructor)]
pub struct PipelineWorker {
    repo: Arc<dyn PipelineRepository>,
    rx_pipeline: mpsc::Receiver<PipelineMessage>,
}

impl BackgroundWorker for PipelineWorker {
    fn spawn_worker(mut self, mut shutdown: Receiver<bool>) -> JoinHandle<()> {
        tokio::spawn(async move {
            'main: loop {
                tokio::select! {
                    _ = shutdown.changed() => {
                        if *shutdown.borrow() {
                            break 'main;
                        }
                    }
                    Some(mes) = self.rx_pipeline.recv() => {
                        self.handle_message(mes).await;
                    }
                }
            }
        })
    }
}

impl PipelineWorker {
    async fn handle_message(&self, message: PipelineMessage) {
        match message {
            PipelineMessage::GetPipeline { id, respond_tx } => {
                let _ = respond_tx.send(self.repo.get_pipeline(id).await);
            }
        }
    }
}
