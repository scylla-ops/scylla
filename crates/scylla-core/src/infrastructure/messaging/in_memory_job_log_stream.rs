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

/// Beyond this a lagging subscriber drops lines; the persisted snapshot stays authoritative.
const CHANNEL_CAPACITY: usize = 8192;

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
        // Poisoning is recovered: plain data, and a panic here would take down log fan-out.
        let mut map = self
            .channels
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.entry(job_id.to_string())
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .clone()
    }

    /// Never creates one: a late subscriber must not resurrect the channel of a finished job.
    fn existing_sender(&self, job_id: &str) -> Option<broadcast::Sender<JobLog>> {
        self.channels
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(job_id)
            .cloned()
    }

    pub fn open(&self, job_id: &str) {
        let _ = self.sender_for(job_id);
    }

    pub fn publish(&self, log: JobLog) {
        let _ = self.sender_for(log.job_id().as_str()).send(log);
    }

    /// Without this the map grows by one entry per job ever streamed.
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

        let Some(sender) = self.existing_sender(job_id.as_str()) else {
            return Ok(Box::pin(ReceiverStream::new(rx)));
        };
        let mut receiver = sender.subscribe();
        let node_filter = node_id.cloned();

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
