use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Streaming;
use tonic::Request;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tracing::{error, info, warn};

use scylla_core::domain::entities::PipelineNode;
use scylla_core::domain::value_objects::pipeline::NodeId;
use scylla_protocol::services::app::IssueTokenRequest;
use scylla_protocol::services::app::app_auth_service_client::AppAuthServiceClient;
use scylla_protocol::services::agent::agent_service_client::AgentServiceClient;
use scylla_protocol::services::agent::{AgentDown, AgentNode, AgentUp, agent_down};

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::executor::Executor;

pub struct Agent {
    config: AgentConfig,
}

impl Agent {
    #[must_use]
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    /// Borrow the underlying agent configuration.
    #[must_use]
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Connect to the control plane and serve dispatched jobs, reconnecting with
    /// backoff when the stream drops (e.g. a revoked grant closes it, or the
    /// control plane restarts). Gives up after `max_reconnect_attempts`
    /// consecutive connection failures (`0` = forever); the counter resets on
    /// every successful connection.
    pub async fn run(&self) -> Result<(), AgentError> {
        let backoff = Duration::from_secs(self.config.reconnect_backoff_secs);
        let max = self.config.max_reconnect_attempts;
        let mut failures: u32 = 0;

        loop {
            match self.connect().await {
                Ok((inbound, up_tx)) => {
                    failures = 0;
                    info!("agent stream open — waiting for jobs");
                    self.serve(inbound, up_tx).await;
                    info!("agent stream closed; reconnecting");
                }
                Err(e) => {
                    failures += 1;
                    warn!(error = %e, attempt = failures, max, "failed to connect to control plane");
                    if max != 0 && failures >= max {
                        error!("giving up after {failures} connection attempts");
                        return Err(e);
                    }
                }
            }
            tokio::time::sleep(backoff).await;
        }
    }

    /// Build a channel, exchange credentials for a bearer token, and open the
    /// agent stream. Returns the inbound (server→agent) stream and the
    /// up-stream sender (agent→server).
    async fn connect(&self) -> Result<(Streaming<AgentDown>, mpsc::Sender<AgentUp>), AgentError> {
        let channel = Channel::from_shared(self.config.control_plane_url.clone())
            .map_err(|e| AgentError::InvalidUrl {
                url: self.config.control_plane_url.clone(),
                message: e.to_string(),
            })?
            .connect()
            .await?;

        let token = AppAuthServiceClient::new(channel.clone())
            .issue_token(IssueTokenRequest {
                app_id: self.config.app_id.clone(),
                secret: self.config.app_secret.clone(),
            })
            .await?
            .into_inner()
            .token;

        let buffer = usize::try_from(self.config.publish_buffer_size).unwrap_or(8192);
        let (up_tx, up_rx) = mpsc::channel::<AgentUp>(buffer);
        let bearer: MetadataValue<_> = format!("Bearer {token}")
            .parse()
            .map_err(|_| AgentError::InvalidToken("token is not valid header ASCII".into()))?;
        let mut request = Request::new(ReceiverStream::new(up_rx));
        request.metadata_mut().insert("authorization", bearer);

        let inbound = AgentServiceClient::new(channel)
            .open(request)
            .await?
            .into_inner();
        Ok((inbound, up_tx))
    }

    /// Run dispatched jobs until the stream ends (clean close or error).
    async fn serve(&self, mut inbound: Streaming<AgentDown>, up_tx: mpsc::Sender<AgentUp>) {
        loop {
            match inbound.message().await {
                Ok(Some(down)) => match down.payload {
                    Some(agent_down::Payload::Dispatch(dispatch)) => {
                        info!(
                            job_id = %dispatch.job_id,
                            pipeline_id = %dispatch.pipeline_id,
                            nodes = dispatch.nodes.len(),
                            "received job"
                        );
                        let nodes = match to_domain_nodes(dispatch.nodes) {
                            Ok(nodes) => nodes,
                            Err(e) => {
                                warn!(job_id = %dispatch.job_id, error = %e, "invalid dispatch nodes, skipping");
                                continue;
                            }
                        };
                        let executor = Executor::new(up_tx.clone(), dispatch.job_id.clone());
                        // V1: sequential — finish the job before accepting the next.
                        if let Err(e) = executor.run(nodes).await {
                            warn!(job_id = %dispatch.job_id, error = %e, "job execution failed");
                        }
                    }
                    None => {}
                },
                Ok(None) => break,
                Err(e) => {
                    warn!(error = %e, "agent stream error");
                    break;
                }
            }
        }
    }
}

fn to_domain_nodes(nodes: Vec<AgentNode>) -> Result<Vec<PipelineNode>, String> {
    nodes
        .into_iter()
        .map(|n| {
            let id = NodeId::new(&n.node_id).map_err(|e| e.to_string())?;
            let deps = n
                .deps
                .iter()
                .map(|d| NodeId::new(d).map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            PipelineNode::new(id, deps, n.command, n.args).map_err(|e| e.to_string())
        })
        .collect()
}
