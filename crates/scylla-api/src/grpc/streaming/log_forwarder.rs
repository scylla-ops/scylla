//! Forwards a domain `JobLogLiveStream` onto a bounded mpsc, exposed as a
//! `ReceiverStream` for tonic. Back-pressure drops lines rather than blocking
//! the upstream; the task exits cleanly when the client disconnects.

use crate::grpc::mappers::{domain_error_to_status, job_log_to_proto};
use futures_util::StreamExt;
use scylla_core::application::JobLogLiveStream;
use scylla_protocol::services::job::JobLogEvent;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::{info, warn};

const FORWARD_CHANNEL_CAPACITY: usize = 256;

/// Spawn a forwarder task and return a tonic-ready stream.
///
/// The task terminates when the upstream ends, the receiver is dropped
/// (client disconnect), or an unrecoverable transport error is observed.
#[must_use]
pub fn spawn_log_forwarder(
    stream: JobLogLiveStream,
) -> ReceiverStream<Result<JobLogEvent, Status>> {
    let (tx, rx) = mpsc::channel::<Result<JobLogEvent, Status>>(FORWARD_CHANNEL_CAPACITY);
    tokio::spawn(forward(stream, tx));
    ReceiverStream::new(rx)
}

async fn forward(mut stream: JobLogLiveStream, tx: mpsc::Sender<Result<JobLogEvent, Status>>) {
    info!("log forwarder: stream opened");
    let mut forwarded = 0_u64;
    loop {
        tokio::select! {
            item = stream.next() => match item {
                Some(Ok(log)) => {
                    let evt = JobLogEvent { log: Some(job_log_to_proto(&log)) };
                    match tx.try_send(Ok(evt)) {
                        Ok(()) => forwarded += 1,
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            warn!("log forwarder back-pressure: dropping line");
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => break,
                    }
                }
                Some(Err(e)) => {
                    // Route through the central mapper: correct gRPC code per
                    // variant and internal detail suppressed (don't leak `{e}`).
                    let _ = tx.send(Err(domain_error_to_status(e))).await;
                }
                None => {
                    info!("log forwarder: upstream stream ended");
                    break;
                }
            },
            () = tx.closed() => {
                info!(forwarded, "log forwarder: client disconnected");
                break;
            }
        }
    }
    info!(forwarded, "log forwarder task ended");
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use futures_util::stream;
    use scylla_core::domain::entities::{JobId, JobLog};
    use scylla_core::domain::errors::DomainResult;
    use scylla_core::domain::value_objects::job::LogStream;
    use scylla_core::domain::value_objects::pipeline::NodeId;

    fn make_log(line: &str) -> JobLog {
        JobLog::new(
            JobId::new("job-1"),
            NodeId::new("step-a").unwrap(),
            LogStream::Stdout,
            line.to_string(),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn forwards_every_upstream_item() {
        let logs: Vec<DomainResult<JobLog>> =
            (0..3).map(|i| Ok(make_log(&format!("line-{i}")))).collect();
        let upstream: JobLogLiveStream = Box::pin(stream::iter(logs));

        let mut out = spawn_log_forwarder(upstream);
        let mut received = Vec::new();
        while let Some(item) = out.next().await {
            received.push(item.expect("forwarder produced an error"));
        }
        assert_eq!(received.len(), 3);
    }

    #[tokio::test]
    async fn surfaces_upstream_errors_as_status() {
        let upstream: JobLogLiveStream = Box::pin(stream::iter([Err(
            scylla_core::domain::errors::DomainError::Internal("boom".into()),
        )]));

        let mut out = spawn_log_forwarder(upstream);
        let first = out.next().await.expect("expected at least one item");
        assert!(first.is_err(), "domain error should surface as a Status");
    }
}
