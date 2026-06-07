use crate::application::{JobLogLiveStream, JobLogStreamPort};
use crate::domain::entities::{JobId, JobLog};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::pipeline::NodeId;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;

/// Per-job live broadcast + bridge buffer. Sized to absorb a fast burst of log
/// lines before a (possibly slow) reader drains them: beyond this a lagging
/// subscriber drops lines, but the persisted snapshot replayed by
/// `JobLogStreamUseCase` remains authoritative, so the view is still complete on
/// (re)open. Generous because a noisy job can emit thousands of lines in a burst.
const CHANNEL_CAPACITY: usize = 8192;

/// In-process job-log fan-out that replaces the broker live-tail. The agent
/// stream handler publishes each log line as it persists it; readers subscribe
/// per job. One bounded broadcast channel per job — a slow reader drops lagged
/// lines (the persisted snapshot via `JobLogRepository` remains authoritative).
#[derive(Default)]
pub struct InMemoryJobLogStream {
    channels: Mutex<HashMap<String, broadcast::Sender<JobLog>>>,
}

impl InMemoryJobLogStream {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn sender_for(&self, job_id: &str) -> broadcast::Sender<JobLog> {
        // Recover from a poisoned lock rather than panicking: the map is plain
        // data with no broken invariant after a thread panic, and a poisoned
        // panic here would take down log fan-out for the whole control plane.
        let mut map = self
            .channels
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.entry(job_id.to_string())
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .clone()
    }

    /// Publish a log line to the live subscribers of its job (no-op if none).
    pub fn publish(&self, log: JobLog) {
        let _ = self.sender_for(log.job_id().as_str()).send(log);
    }

    /// Drop a job's live channel once the job reaches a terminal state. No more
    /// lines will be published, so subscribers receive `Closed` and fall back to
    /// the persisted snapshot (authoritative). Without this the channel map grows
    /// monotonically — one entry per job ever streamed — on a long-running
    /// control plane.
    pub fn close(&self, job_id: &str) {
        let mut map = self
            .channels
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.remove(job_id);
    }
}

#[async_trait]
impl JobLogStreamPort for InMemoryJobLogStream {
    async fn subscribe(
        &self,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<JobLogLiveStream> {
        let mut receiver = self.sender_for(job_id.as_str()).subscribe();
        let node_filter = node_id.cloned();
        let (tx, rx) = mpsc::channel::<DomainResult<JobLog>>(CHANNEL_CAPACITY);

        // Bridge the broadcast receiver to an mpsc the caller can stream. Lagged
        // lines are skipped (the historical snapshot covers any gap); the task
        // ends when the broadcast closes or the reader drops the stream.
        tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(log) => {
                        if node_filter.as_ref().is_none_or(|n| log.node_id() == n)
                            && tx.send(Ok(log)).await.is_err()
                        {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}
