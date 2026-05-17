use hermes_broker_client::Subscriber;
use scylla_core::application::JobUseCases;
use scylla_core::domain::entities::JobId;
use scylla_core::domain::value_objects::job::NodeState;
use scylla_core::domain::value_objects::job::{JobEvent, JobStatusUpdate};
use scylla_core::domain::value_objects::pipeline::NodeId;
use scylla_core::infrastructure::SurrealJobRepository;
use std::sync::Arc;
use tonic::transport::Channel;
use tracing::{error, info, warn};

pub async fn run(channel: Channel, job_uc: Arc<JobUseCases<SurrealJobRepository>>) {
    let result: anyhow::Result<()> = async {
        let mut subscriber = Subscriber::new(channel)
            .await
            .map_err(|e| anyhow::anyhow!("failed to create subscriber: {e}"))?;

        subscriber
            .subscribe("scylla.jobs.status.>", None)
            .await
            .map_err(|e| anyhow::anyhow!("failed to subscribe: {e}"))?;

        info!("subscribed to scylla.jobs.status.>");

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
    .await;

    if let Err(e) = result {
        error!(error = %e, "status listener failed");
    }
}
