//! Forwards a domain `JobLogLiveStream` onto a bounded mpsc, exposed as a
//! `ReceiverStream` for tonic. Applies back-pressure (awaits a slot) rather than
//! dropping lines, so the full history replay + live tail reach the client
//! intact; the task exits cleanly when the client disconnects.

use crate::application::JobLogLiveStream;
use crate::grpc::mappers::{domain_error_to_status, job_log_to_proto};
use futures_util::StreamExt;
use scylla_protocol::job::v1::TailJobLogsResponse;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::info;

const FORWARD_CHANNEL_CAPACITY: usize = 256;

/// Spawn a forwarder task and return a tonic-ready stream.
///
/// The task terminates when the upstream ends, the receiver is dropped
/// (client disconnect), or an unrecoverable transport error is observed.
#[must_use]
pub fn spawn_log_forwarder(
    stream: JobLogLiveStream,
) -> ReceiverStream<Result<TailJobLogsResponse, Status>> {
    let (tx, rx) = mpsc::channel::<Result<TailJobLogsResponse, Status>>(FORWARD_CHANNEL_CAPACITY);
    tokio::spawn(forward(stream, tx));
    ReceiverStream::new(rx)
}

async fn forward(
    mut stream: JobLogLiveStream,
    tx: mpsc::Sender<Result<TailJobLogsResponse, Status>>,
) {
    info!("log forwarder: stream opened");
    let mut forwarded = 0_u64;
    loop {
        tokio::select! {
            item = stream.next() => match item {
                Some(Ok(log)) => {
                    let evt = TailJobLogsResponse { log: Some(job_log_to_proto(&log)) };
                    // Await a slot (back-pressure) instead of dropping: a noisy
                    // job's full log must reach the client, not just the first
                    // bufferful. If the client has gone, `send` errors -> stop.
                    if tx.send(Ok(evt)).await.is_err() {
                        break;
                    }
                    forwarded += 1;
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
    use scylla_core::domain::errors::DomainResult;
    use scylla_core::domain::ids::JobId;
    use scylla_core::domain::job::JobLog;
    use scylla_core::domain::job::LogStream;
    use scylla_core::domain::pipeline::NodeId;

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
