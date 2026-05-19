use crate::application::{JobLogLiveStream, JobLogStreamPort};
use crate::domain::entities::{JobId, JobLog};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::job::JobLogLine;
use crate::domain::value_objects::pipeline::NodeId;
use async_trait::async_trait;
use hermes_broker_client::Subscriber;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use tracing::{info, warn};

/// Hermes broker-backed adapter that subscribes to `scylla.jobs.logs.{job_id}.{node_id}`
/// (or `scylla.jobs.logs.{job_id}.>` when no node filter is given) and yields a
/// stream of [`JobLog`] domain entities.
pub struct HermesJobLogStream {
    channel: Channel,
}

impl HermesJobLogStream {
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

#[async_trait]
impl JobLogStreamPort for HermesJobLogStream {
    async fn subscribe(
        &self,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<JobLogLiveStream> {
        let subject = match node_id {
            Some(nid) => format!("scylla.jobs.logs.{job_id}.{nid}"),
            None => format!("scylla.jobs.logs.{job_id}.>"),
        };

        let mut subscriber = Subscriber::new(self.channel.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("broker connect: {e}")))?;
        subscriber
            .subscribe(&subject, None)
            .await
            .map_err(|e| DomainError::infrastructure(format!("broker subscribe: {e}")))?;

        info!(subject = %subject, "hermes job log stream opened");

        let job_id = job_id.clone();
        let (tx, rx) = mpsc::channel::<DomainResult<JobLog>>(256);

        tokio::spawn(async move {
            while let Some(msg) = subscriber.recv().await {
                if tx.is_closed() {
                    break;
                }
                let log_line: JobLogLine = match serde_json::from_slice(&msg.payload) {
                    Ok(line) => line,
                    Err(e) => {
                        warn!(error = %e, "malformed JobLogLine payload, skipping");
                        continue;
                    }
                };

                let node_id = match NodeId::new(&log_line.node_id) {
                    Ok(nid) => nid,
                    Err(e) => {
                        warn!(error = %e, node_id = %log_line.node_id, "invalid node_id, skipping");
                        continue;
                    }
                };

                let timestamp = chrono::DateTime::parse_from_rfc3339(&log_line.timestamp)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc));

                let log = JobLog::new(
                    job_id.clone(),
                    node_id,
                    log_line.stream,
                    log_line.line,
                    timestamp,
                );

                if tx.send(Ok(log)).await.is_err() {
                    break;
                }
            }
            info!("hermes job log stream closed");
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}
