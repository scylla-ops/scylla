use crate::api::grpc::BackgroundWorker;
use crate::api::grpc::pipeline::snapshot::models::PipelineSnapshotRecord;
use crate::api::grpc::pipeline::snapshot::service::PipelineSnapshotService;
use derive_more::Constructor;
use std::sync::Arc;
use tokio::sync::watch::Receiver;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use uuid::Uuid;

pub enum PipelineSnapshotMessage {
    CreateSnapshot {
        pipeline_id: Uuid,
        tx: oneshot::Sender<anyhow::Result<PipelineSnapshotRecord>>,
    },
    ListSnapshots {
        pipeline_id: Uuid,
        tx: oneshot::Sender<anyhow::Result<Vec<PipelineSnapshotRecord>>>,
    },
}

#[derive(Constructor)]
pub struct PipelineSnapshotWorker {
    service: Arc<PipelineSnapshotService>,
    rx: mpsc::Receiver<PipelineSnapshotMessage>,
}

impl BackgroundWorker for PipelineSnapshotWorker {
    fn spawn_worker(mut self, mut shutdown: Receiver<bool>) -> JoinHandle<()> {
        tokio::spawn(async move {
            'main: loop {
                tokio::select! {
                    _ = shutdown.changed() => {
                        if *shutdown.borrow() {
                            break 'main;
                        }
                    }
                    Some(mes) = self.rx.recv() => {
                        self.handle_message(mes).await;
                    }
                }
            }
        })
    }
}

impl PipelineSnapshotWorker {
    async fn handle_message(&self, message: PipelineSnapshotMessage) {
        use PipelineSnapshotMessage as M;
        match message {
            M::ListSnapshots { pipeline_id, tx } => {
                let result = self
                    .service
                    .list_snapshots(pipeline_id)
                    .await
                    .map_err(|e| anyhow::anyhow!(e));
                let _ = tx.send(result);
            }
            M::CreateSnapshot { pipeline_id, tx } => {
                let result = self
                    .service
                    .create_snapshot(pipeline_id)
                    .await
                    .map_err(|e| anyhow::anyhow!(e));

                match result {
                    Ok(snapshot_id) => {
                        let snapshot = self
                            .service
                            .get_snapshot(snapshot_id)
                            .await
                            .map_err(|e| anyhow::anyhow!(e));
                        let _ = tx.send(snapshot);
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                    }
                }
            }
        }
    }
}
