use derive_more::Constructor;
use protocol::job::Job;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::Sender;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

pub mod service;

#[derive(Constructor, Clone, Default)]
pub struct Orchestrator {
    worker_registry: WorkerRegistry,
}

#[derive(Debug)]
struct WorkerRecord {
    last_seen: Instant,
    tx_job: Option<mpsc::Sender<Job>>,
}

#[derive(Default, Clone, Debug)]
pub struct WorkerRegistry {
    inner: Arc<RwLock<HashMap<Uuid, WorkerRecord>>>,
}

impl WorkerRegistry {
    async fn register(&self) -> Uuid {
        let id = Uuid::new_v4();
        let rec = WorkerRecord {
            last_seen: Instant::now(),
            tx_job: None,
        };
        self.inner.write().await.insert(id, rec);
        id
    }

    async fn unregister(&self, id: Uuid) {
        self.inner.write().await.remove(&id);
    }

    async fn attach_stream(
        &self,
        id: Uuid,
        tx_job: tokio::sync::mpsc::Sender<Job>,
    ) -> Result<(), String> {
        self.inner
            .write()
            .await
            .get_mut(&id)
            .map(|entry| entry.tx_job = Some(tx_job))
            .ok_or_else(|| format!("Worker id {} not found", id))
    }

    pub async fn get_first_available(&self) -> Option<Sender<Job>> {
        let workers = self.inner.read().await;
        workers.iter().find_map(|(_id, rec)| rec.tx_job.clone())
    }
}
