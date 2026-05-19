use hermes_broker_client::Subscriber;
use hermes_broker_proto::PublishRequest;
use hermes_broker_proto::broker_client::BrokerClient;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use tracing::{error, info, warn};

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::executor::Executor;
use scylla_core::domain::value_objects::pipeline::JobDispatch;

pub struct Agent {
    config: AgentConfig,
    channel: Channel,
}

impl Agent {
    /// Connect to the Hermes broker.
    pub async fn connect(config: AgentConfig) -> Result<Self, AgentError> {
        let channel = Channel::from_shared(config.broker_url.clone())
            .map_err(|e| AgentError::InvalidBrokerUrl {
                url: config.broker_url.clone(),
                message: e.to_string(),
            })?
            .connect()
            .await?;

        info!("connected to broker at {}", config.broker_url);
        Ok(Self { config, channel })
    }

    /// Cloned broker channel for sharing with auxiliary publishers (presence, etc.).
    #[must_use]
    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    /// Borrow the underlying agent configuration.
    #[must_use]
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Main loop: subscribe to dispatch subject, receive jobs, execute them.
    pub async fn run(&self) -> Result<(), AgentError> {
        // Create subscriber for receiving job dispatches
        let mut subscriber = Subscriber::new(self.channel.clone())
            .await
            .map_err(AgentError::Send)?;

        subscriber
            .subscribe(
                &self.config.dispatch_subject,
                Some(self.config.queue_group.clone()),
            )
            .await
            .map_err(|_| AgentError::StreamClosed)?;

        info!(
            subject = %self.config.dispatch_subject,
            queue_group = %self.config.queue_group,
            "subscribed — waiting for jobs"
        );

        // Open a raw publish stream for status updates and logs.
        // We use the raw BrokerClient so we can set reply_to on messages if needed.
        // INVARIANT: publish_buffer_size is a clap-validated u32 >= 1 and always fits in usize.
        let publish_buffer = usize::try_from(self.config.publish_buffer_size)
            .expect("publish_buffer_size fits in usize (clap range starts at 1)");
        let (publish_tx, publish_rx) = mpsc::channel::<PublishRequest>(publish_buffer);
        let mut publish_client = BrokerClient::new(self.channel.clone());
        tokio::spawn(async move {
            match publish_client
                .publish(ReceiverStream::new(publish_rx))
                .await
            {
                Ok(resp) => {
                    let ack = resp.into_inner();
                    info!(total = ack.total_published, "publish stream completed");
                }
                Err(e) => {
                    warn!(error = %e, "publish stream failed");
                }
            }
        });

        // Receive loop
        while let Some(msg) = subscriber.recv().await {
            let dispatch: JobDispatch = match serde_json::from_slice(&msg.payload) {
                Ok(d) => d,
                Err(e) => {
                    error!(error = %e, "failed to deserialize dispatch message, skipping");
                    continue;
                }
            };

            // Read reply_to from the message
            let status_subject = if msg.reply_to.is_empty() {
                warn!("no reply_to in dispatch message, using default");
                format!("scylla.jobs.status.{}", dispatch.job_id)
            } else {
                msg.reply_to
            };

            info!(
                job_id = %dispatch.job_id,
                pipeline_id = %dispatch.pipeline_id,
                nodes = dispatch.nodes.len(),
                reply_to = %status_subject,
                "received job"
            );

            let executor =
                Executor::new(publish_tx.clone(), status_subject, dispatch.job_id.clone());

            // V1: sequential execution — wait for job to finish before accepting next
            if let Err(e) = executor.run(dispatch.nodes).await {
                warn!(job_id = %dispatch.job_id, error = %e, "job execution failed");
            }
        }

        info!("dispatch stream closed, shutting down");
        Ok(())
    }
}
