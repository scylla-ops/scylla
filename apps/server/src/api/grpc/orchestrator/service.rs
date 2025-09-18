use crate::api::grpc::job::models::ExecutionStatus;
use crate::api::grpc::job::worker::JobMessage;
use derive_more::Constructor;
use protocol::job::Job;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

type JobSender = Sender<Job>;

#[derive(Constructor)]
pub struct OrchestratorService {
    workers_queue: Arc<RwLock<VecDeque<WorkerRecord>>>,

    // Channels
    tx_job: mpsc::Sender<JobMessage>,
}

#[derive(Debug)]
pub struct WorkerRecord {
    pub id: Uuid,
    pub tx_job: JobSender,
}

impl OrchestratorService {
    pub async fn queue_worker(&self, id: Uuid, job: JobSender) {
        self.workers_queue
            .write()
            .await
            .push_back(WorkerRecord { id, tx_job: job });
    }

    pub async fn get_first_available(&self) -> anyhow::Result<WorkerRecord> {
        let mut workers = self.workers_queue.write().await;
        workers
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("No workers available"))
    }

    pub async fn shutdown(&self) {
        let mut workers = self.workers_queue.write().await;
        workers.clear();
    }

    pub async fn update_job(&self, job_id: Uuid, new_status: ExecutionStatus) {
        let _ = self
            .tx_job
            .send(JobMessage::UpdateJob { job_id, new_status })
            .await;
    }

    pub async fn update_stage(&self, stage_id: Uuid, new_status: ExecutionStatus) {
        let _ = self
            .tx_job
            .send(JobMessage::UpdateStage {
                stage_id,
                new_status,
            })
            .await;
    }

    pub async fn update_step(&self, step_id: Uuid, new_status: ExecutionStatus) {
        let _ = self
            .tx_job
            .send(JobMessage::UpdateStep {
                step_id,
                new_status,
            })
            .await;
    }
}
