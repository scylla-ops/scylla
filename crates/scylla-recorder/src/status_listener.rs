use hermes_broker_client::Subscriber;
use scylla_core::application::JobUseCases;
use scylla_core::domain::entities::JobId;
use scylla_core::domain::value_objects::job::NodeState;
use scylla_core::domain::value_objects::job::{JobEvent, JobStatusUpdate};
use scylla_core::domain::value_objects::pipeline::NodeId;
use scylla_core::infrastructure::PgJobRepository;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tonic::transport::Channel;
use tracing::{error, info, warn};

const STATUS_SUBJECT: &str = "scylla.jobs.status.>";
const RECONNECT_BACKOFF: StdDuration = StdDuration::from_secs(2);

pub async fn run(channel: Channel, job_uc: Arc<JobUseCases<PgJobRepository>>) {
    loop {
        if let Err(e) = status_once(channel.clone(), &job_uc).await {
            warn!(error = %e, "status listener exited; reconnecting");
        }
        tokio::time::sleep(RECONNECT_BACKOFF).await;
    }
}

async fn status_once(
    channel: Channel,
    job_uc: &JobUseCases<PgJobRepository>,
) -> anyhow::Result<()> {
    let mut subscriber = Subscriber::new(channel)
        .await
        .map_err(|e| anyhow::anyhow!("subscriber: {e}"))?;
    subscriber
        .subscribe(STATUS_SUBJECT, None)
        .await
        .map_err(|e| anyhow::anyhow!("subscribe: {e}"))?;

    info!(subject = STATUS_SUBJECT, "subscribed");

    while let Some(msg) = subscriber.recv().await {
        let update: JobStatusUpdate = match serde_json::from_slice(&msg.payload) {
            Ok(u) => u,
            Err(e) => {
                warn!(error = %e, "failed to deserialize status update, skipping");
                continue;
            }
        };

        let job_id = JobId::new(&update.job_id);

        let mut job = match job_uc.get(&job_id).await {
            Ok(j) => j,
            Err(e) => {
                warn!(job_id = %update.job_id, error = %e, "failed to load job");
                continue;
            }
        };

        let result = match update.event {
            JobEvent::JobStarted => job.start(),
            JobEvent::NodeStarted { ref node_id } => match NodeId::new(node_id) {
                Ok(nid) => job.apply_node_started(&nid, chrono::Utc::now()),
                Err(e) => Err(e),
            },
            JobEvent::NodeCompleted { ref node_id } => match NodeId::new(node_id) {
                Ok(nid) => {
                    job.apply_node_finished(&nid, NodeState::Completed, chrono::Utc::now())
                }
                Err(e) => Err(e),
            },
            JobEvent::NodeFailed { ref node_id, .. } => match NodeId::new(node_id) {
                Ok(nid) => {
                    job.apply_node_finished(&nid, NodeState::Failed, chrono::Utc::now())
                }
                Err(e) => Err(e),
            },
            JobEvent::NodeSkipped { ref node_id } => match NodeId::new(node_id) {
                Ok(nid) => job.apply_node_skipped(&nid, chrono::Utc::now()),
                Err(e) => Err(e),
            },
            JobEvent::JobCompleted => job.complete(),
            JobEvent::JobFailed { .. } => job.fail(),
        };

        if let Err(e) = result {
            warn!(job_id = %update.job_id, event = ?update.event, error = %e, "failed to apply status event");
            continue;
        }

        if let Err(e) = job_uc.update(&job).await {
            error!(job_id = %update.job_id, error = %e, "failed to persist job");
        }
    }

    info!("status listener stream closed");
    Ok(())
}
