use crate::api::grpc::BackgroundWorker;
use crate::api::grpc::job::models::ExecutionStatus;
use crate::api::grpc::job::service::JobService;
use derive_more::Constructor;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::warn;
use uuid::Uuid;

#[derive(Constructor)]
pub struct JobWorker {
    service: Arc<JobService>,
    rx_orchestrator: mpsc::Receiver<JobMessage>,
}

#[derive(Debug)]
pub enum JobMessage {
    UpdateJob {
        job_id: Uuid,
        new_status: ExecutionStatus,
    },
    UpdateStage {
        stage_id: Uuid,
        new_status: ExecutionStatus,
    },
    UpdateStep {
        step_id: Uuid,
        new_status: ExecutionStatus,
    },
}

impl BackgroundWorker for JobWorker {
    fn spawn_worker(mut self, mut shutdown: tokio::sync::watch::Receiver<bool>) -> JoinHandle<()> {
        tokio::spawn(async move {
            'main: loop {
                tokio::select! {
                    _ = shutdown.changed() => {
                        if *shutdown.borrow() {
                            warn!("Job worker shutdown");
                            break 'main;
                        }
                    }
                    Some(mes) = self.rx_orchestrator.recv() => {
                        self.handle_message(mes).await;
                    }
                }
            }
        })
    }
}

impl JobWorker {
    async fn handle_message(&self, message: JobMessage) {
        match message {
            JobMessage::UpdateJob { job_id, new_status } => {
                let _ = self.service.update_job(job_id, new_status).await;
            }
            JobMessage::UpdateStage {
                stage_id,
                new_status,
            } => {
                let _ = self.service.update_stage(stage_id, new_status).await;
            }
            JobMessage::UpdateStep {
                step_id,
                new_status,
            } => {
                let _ = self.service.update_step(step_id, new_status).await;
            }
        }
    }
}
