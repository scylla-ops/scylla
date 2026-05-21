use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tracing::{info, warn};

use scylla_core::domain::entities::PipelineNode;
use scylla_core::domain::value_objects::pipeline::NodeId;
use scylla_protocol::services::app::IssueTokenRequest;
use scylla_protocol::services::app::app_auth_service_client::AppAuthServiceClient;
use scylla_protocol::services::worker::worker_service_client::WorkerServiceClient;
use scylla_protocol::services::worker::{WorkerNode, WorkerUp, worker_down};

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::executor::Executor;

pub struct Agent {
    config: AgentConfig,
    channel: Channel,
    token: String,
}

impl Agent {
    /// Connect to the control plane and exchange the app credentials for a
    /// bearer token used on the worker stream.
    pub async fn connect(config: AgentConfig) -> Result<Self, AgentError> {
        let channel = Channel::from_shared(config.control_plane_url.clone())
            .map_err(|e| AgentError::InvalidUrl {
                url: config.control_plane_url.clone(),
                message: e.to_string(),
            })?
            .connect()
            .await?;
        info!(url = %config.control_plane_url, "connected to control plane");

        let mut auth = AppAuthServiceClient::new(channel.clone());
        let token = auth
            .issue_token(IssueTokenRequest {
                app_id: config.app_id.clone(),
                secret: config.app_secret.clone(),
            })
            .await?
            .into_inner()
            .token;
        info!(app_id = %config.app_id, "obtained app token");

        Ok(Self {
            config,
            channel,
            token,
        })
    }

    /// Borrow the underlying agent configuration.
    #[must_use]
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Open the worker stream and execute dispatched jobs until it closes.
    pub async fn run(&self) -> Result<(), AgentError> {
        let buffer = usize::try_from(self.config.publish_buffer_size).unwrap_or(8192);
        let (up_tx, up_rx) = mpsc::channel::<WorkerUp>(buffer);

        let bearer: MetadataValue<_> = format!("Bearer {}", self.token)
            .parse()
            .map_err(|_| AgentError::InvalidToken("token is not valid header ASCII".into()))?;
        let mut request = Request::new(ReceiverStream::new(up_rx));
        request.metadata_mut().insert("authorization", bearer);

        let mut client = WorkerServiceClient::new(self.channel.clone());
        let mut inbound = client.open(request).await?.into_inner();
        info!("worker stream open — waiting for jobs");

        while let Some(down) = inbound.message().await? {
            match down.payload {
                Some(worker_down::Payload::Dispatch(dispatch)) => {
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
            }
        }

        info!("worker stream closed, shutting down");
        Ok(())
    }
}

fn to_domain_nodes(nodes: Vec<WorkerNode>) -> Result<Vec<PipelineNode>, String> {
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
