use crate::api::grpc::job::repos::surreal::JobRepositorySurreal;
use crate::api::grpc::job::service::JobService;
use crate::api::grpc::pipeline::repos::surreal::PipelineRepositorySurreal;
use crate::api::grpc::pipeline::snapshot::repos::surreal::PipelineSnapshotRepositorySurreal;
use crate::api::grpc::utils::Id;
use derive_more::Constructor;
use protocol::job::{JobData, JobEntry};
use std::collections::VecDeque;
use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;
use tokio::sync::mpsc::Sender;
use tracing::warn;
use uuid::Uuid;

type JobSender = Sender<JobEntry>;

#[derive(Constructor)]
pub struct OrchestratorService {
    workers_queue: Arc<RwLock<VecDeque<WorkerRecord>>>,
}

pub static ORCHESTRATOR_SERVICE: LazyLock<Arc<OrchestratorService>> = LazyLock::new(|| {
    Arc::new(OrchestratorService::new(Arc::new(RwLock::new(
        VecDeque::new(),
    ))))
});

#[derive(Debug)]
pub struct WorkerRecord {
    pub _id: Uuid,
    pub tx_job: JobSender,
}

impl OrchestratorService {
    pub async fn queue_job(&self, job: JobEntry) {
        if let Ok(WorkerRecord { tx_job, .. }) = self.get_first_available().await {
            let _ = tx_job.send(job).await;
        } else {
            warn!("No worker available to handle job")
        }
    }

    pub async fn queue_worker(&self, id: Uuid, job: JobSender) {
        self.workers_queue.write().await.push_back(WorkerRecord {
            _id: id,
            tx_job: job,
        });
    }

    pub async fn get_first_available(&self) -> anyhow::Result<WorkerRecord> {
        let mut workers = self.workers_queue.write().await;
        workers
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("No workers available"))
    }

    pub async fn update_job_data(&self, job_id: Id, job_data: JobData) {
        let _ = JobService::<
            JobRepositorySurreal,
            PipelineRepositorySurreal,
            PipelineSnapshotRepositorySurreal,
        >::update_job(job_id, job_data)
        .await;
    }
}
