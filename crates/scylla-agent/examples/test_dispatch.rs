//! Test client that publishes a fake JobDispatch to the broker,
//! then subscribes to status + log subjects to watch the agent work.
//!
//! Usage:
//!   1. Start the broker:   cargo run -p scylla-broker
//!   2. Start the agent:    cargo run -p scylla-agent -- --broker-url http://127.0.0.1:50052
//!   3. Run this:           cargo run -p scylla-agent --example test_dispatch

use hermes_broker_client::Subscriber;
use hermes_broker_proto::PublishRequest;
use hermes_broker_proto::broker_client::BrokerClient;
use scylla_core::domain::entities::PipelineNode;
use scylla_core::domain::value_objects::pipeline::JobDispatch;
use scylla_core::domain::value_objects::pipeline::NodeId;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};

const BROKER_URL: &str = "http://127.0.0.1:50052";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let channel = hermes_broker_client::connect(BROKER_URL, None).await?;
    info!("connected to broker at {BROKER_URL}");

    let job_id = "test-job-001";

    // Subscribe to status + logs BEFORE publishing
    let mut status_sub = Subscriber::new(channel.clone()).await?;
    status_sub
        .subscribe(format!("scylla.jobs.status.{job_id}"), None)
        .await?;
    info!("subscribed to scylla.jobs.status.{job_id}");

    let mut log_sub = Subscriber::new(channel.clone()).await?;
    log_sub
        .subscribe(format!("scylla.jobs.logs.{job_id}.>"), None)
        .await?;
    info!("subscribed to scylla.jobs.logs.{job_id}.>");

    // Build a test dispatch with a simple DAG:
    //   step-a (no deps) → echo "hello from step-a"
    //   step-b (depends on step-a) → echo "hello from step-b"
    let dispatch = JobDispatch {
        job_id: job_id.to_string(),
        pipeline_id: "test-pipeline-001".to_string(),
        nodes: vec![
            PipelineNode::new(
                NodeId::new("step-a").unwrap(),
                vec![],
                "echo".to_string(),
                vec!["hello from step-a".to_string()],
            )
            .unwrap(),
            PipelineNode::new(
                NodeId::new("step-b").unwrap(),
                vec![NodeId::new("step-a").unwrap()],
                "echo".to_string(),
                vec!["hello from step-b".to_string()],
            )
            .unwrap(),
        ],
    };

    let payload = serde_json::to_vec(&dispatch)?;
    info!(
        "publishing JobDispatch: job_id={}, nodes={}",
        dispatch.job_id,
        dispatch.nodes.len()
    );

    // Publish via raw BrokerClient so we can set reply_to
    let (tx, rx) = tokio::sync::mpsc::channel::<PublishRequest>(16);
    let mut client = BrokerClient::new(channel);
    tokio::spawn(async move {
        match client.publish(ReceiverStream::new(rx)).await {
            Ok(resp) => info!(total = resp.into_inner().total_published, "publish done"),
            Err(e) => warn!(error = %e, "publish failed"),
        }
    });

    tx.send(PublishRequest {
        subject: "scylla.jobs.dispatch".into(),
        payload,
        reply_to: format!("scylla.jobs.status.{job_id}"),
    })
    .await?;

    // Drop sender to close the publish stream
    drop(tx);

    info!("dispatch sent — waiting for events...\n");

    // Listen for status events and logs concurrently
    let status_handle = tokio::spawn(async move {
        while let Some(msg) = status_sub.recv().await {
            let text = String::from_utf8_lossy(&msg.payload);
            info!("[STATUS] {text}");
        }
    });

    let log_handle = tokio::spawn(async move {
        while let Some(msg) = log_sub.recv().await {
            let text = String::from_utf8_lossy(&msg.payload);
            info!("[LOG]    {text}");
        }
    });

    // Wait a bit then exit
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    info!("timeout — done");

    status_handle.abort();
    log_handle.abort();

    Ok(())
}
