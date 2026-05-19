use crate::error::ListenerError;
use hermes_broker_client::Subscriber;
use scylla_core::application::caller::{CallerContext, ServiceIdentity};
use scylla_core::application::{JobLogUseCases, PermissionService};
use scylla_core::domain::entities::{JobId, JobLog};
use scylla_core::domain::value_objects::job::JobLogLine;
use scylla_core::domain::value_objects::pipeline::NodeId;
use scylla_core::infrastructure::PgJobLogRepository;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tonic::transport::Channel;
use tracing::{error, info, warn};

const LOG_SUBJECT: &str = "scylla.jobs.logs.>";
const RECONNECT_BACKOFF: StdDuration = StdDuration::from_secs(2);

pub async fn run<PS: PermissionService + 'static>(
    channel: Channel,
    job_log_uc: Arc<JobLogUseCases<PgJobLogRepository, PS>>,
) {
    loop {
        if let Err(e) = log_once(channel.clone(), &job_log_uc).await {
            warn!(error = %e, "log listener exited; reconnecting");
        }
        tokio::time::sleep(RECONNECT_BACKOFF).await;
    }
}

async fn log_once<PS: PermissionService + 'static>(
    channel: Channel,
    job_log_uc: &JobLogUseCases<PgJobLogRepository, PS>,
) -> Result<(), ListenerError> {
    let caller = CallerContext::Service(ServiceIdentity::recorder());

    let mut subscriber = Subscriber::new(channel)
        .await
        .map_err(|e| ListenerError::SubscriberInit(e.to_string()))?;
    subscriber
        .subscribe(LOG_SUBJECT, None)
        .await
        .map_err(|e| ListenerError::Subscribe {
            subject: LOG_SUBJECT.to_string(),
            message: e.to_string(),
        })?;

    info!(subject = LOG_SUBJECT, "subscribed");

    while let Some(msg) = subscriber.recv().await {
        let log_line: JobLogLine = match serde_json::from_slice(&msg.payload) {
            Ok(l) => l,
            Err(e) => {
                warn!(error = %e, "failed to deserialize log line, skipping");
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

        let Some(job_id_str) = parse_job_id_from_subject(&msg.subject) else {
            warn!(subject = %msg.subject, "unexpected log subject format, skipping");
            continue;
        };

        let timestamp = chrono::DateTime::parse_from_rfc3339(&log_line.timestamp)
            .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc));

        let job_log = JobLog::new(
            JobId::new(job_id_str),
            node_id,
            log_line.stream,
            log_line.line,
            timestamp,
        );

        if let Err(e) = job_log_uc.create(&caller, &job_log).await {
            error!(error = %e, "failed to persist job log");
        }
    }

    info!("log listener stream closed");
    Ok(())
}

/// Parse job_id from subject `scylla.jobs.logs.{job_id}.{node_id}`.
fn parse_job_id_from_subject(subject: &str) -> Option<&str> {
    let rest = subject.strip_prefix("scylla.jobs.logs.")?;
    let job_id = rest.split('.').next()?;
    if job_id.is_empty() {
        return None;
    }
    Some(job_id)
}
