use crate::application::{JobLogLiveStream, JobLogStreamPort};
use crate::domain::errors::DomainResult;
use crate::domain::ids::JobId;
use crate::domain::job::JobLog;
use crate::domain::pipeline::NodeId;
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

    /// The live sender for a job **only if one already exists** — never creates
    /// one. Used by `subscribe`: subscribing must not resurrect a channel for a
    /// job whose live stream was already closed (a finished job), which would
    /// leak an entry that is never published to nor closed again.
    fn existing_sender(&self, job_id: &str) -> Option<broadcast::Sender<JobLog>> {
        self.channels
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(job_id)
            .cloned()
    }

    /// Eagerly create a job's live channel at start, so a reader that subscribes
    /// before the first log line still joins the live stream. Paired with
    /// [`Self::close`] at terminal; since [`JobLogStreamPort::subscribe`] never
    /// creates a channel, a finished job's channel can't be resurrected and
    /// leaked by a late subscriber.
    pub fn open(&self, job_id: &str) {
        let _ = self.sender_for(job_id);
    }

    /// Publish a log line to the live subscribers of its job. Creates the channel
    /// if the job's `open` was missed, so a line is never silently dropped.
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
        let (tx, rx) = mpsc::channel::<DomainResult<JobLog>>(CHANNEL_CAPACITY);

        // No live channel means the job already reached a terminal state (its
        // channel was closed) or never streamed: return an immediately-ending
        // stream so the caller falls back to the persisted snapshot, WITHOUT
        // resurrecting a channel that would then leak forever.
        let Some(sender) = self.existing_sender(job_id.as_str()) else {
            return Ok(Box::pin(ReceiverStream::new(rx)));
        };
        let mut receiver = sender.subscribe();
        let node_filter = node_id.cloned();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::job::LogStream;
    use chrono::Utc;
    use tokio_stream::StreamExt;

    fn log(job: &str) -> JobLog {
        JobLog::new(
            JobId::new(job),
            NodeId::new("n1").unwrap(),
            LogStream::Stdout,
            "hello".to_string(),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn subscribe_to_an_unopened_job_yields_an_empty_stream() {
        let s = InMemoryJobLogStream::new();
        let mut stream = s.subscribe(&JobId::new("ghost"), None).await.unwrap();
        assert!(
            stream.next().await.is_none(),
            "no live channel means an immediately-ending stream (fall back to the snapshot)",
        );
        // Crucially, subscribing did NOT create a channel — otherwise it would
        // leak forever for a job that will never publish or be closed again.
        assert!(
            s.existing_sender("ghost").is_none(),
            "subscribe must not create a channel"
        );
    }

    #[tokio::test]
    async fn open_then_publish_reaches_a_live_subscriber() {
        let s = InMemoryJobLogStream::new();
        s.open("job-1");
        let mut stream = s.subscribe(&JobId::new("job-1"), None).await.unwrap();
        s.publish(log("job-1"));
        let received = stream.next().await.expect("a live line").expect("ok");
        assert_eq!(received.line(), "hello");
    }

    #[tokio::test]
    async fn subscribe_after_close_does_not_resurrect_a_channel() {
        let s = InMemoryJobLogStream::new();
        s.open("job-1");
        s.close("job-1");
        let mut stream = s.subscribe(&JobId::new("job-1"), None).await.unwrap();
        assert!(
            stream.next().await.is_none(),
            "a finished job's live channel must not be recreated",
        );
        assert!(
            s.existing_sender("job-1").is_none(),
            "no leaked channel entry"
        );
    }
}
