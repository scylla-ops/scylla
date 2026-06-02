use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Streaming;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::{Code, Request};
use tracing::{error, info, warn};

use scylla_core::domain::entities::PipelineNode;
use scylla_core::domain::value_objects::pipeline::NodeId;
use scylla_protocol::services::agent::agent_service_client::AgentServiceClient;
use scylla_protocol::services::agent::{AgentDown, AgentNode, AgentUp, agent_down};
use scylla_protocol::services::app::IssueTokenRequest;
use scylla_protocol::services::app::app_auth_service_client::AppAuthServiceClient;

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
    /// exponential backoff when the stream drops (e.g. the control plane
    /// restarts). Gives up after `max_reconnect_attempts` consecutive failures
    /// (`0` = forever).
    ///
    /// Two refinements over a naive retry loop:
    /// - A **terminal** rejection (`Unauthenticated` / `PermissionDenied` /
    ///   `NotFound` — a revoked secret, disabled app, or deleted agent) stops the
    ///   agent immediately. Retrying can't fix bad credentials; doing so would
    ///   hammer the control plane forever.
    /// - The failure counter only resets after a connection that *stayed up*
    ///   (≥ [`MIN_UPTIME_FOR_RESET_SECS`]). A connect-then-instant-close loop
    ///   (server drops us right after open) is counted as a failure so the
    ///   exponential backoff and `max` cap actually engage, instead of a tight
    ///   ~1/s reconnect spin with the counter perpetually reset to zero.
    pub async fn run(&self) -> Result<(), AgentError> {
        let base = self.config.reconnect_backoff_secs;
        let max = self.config.max_reconnect_attempts;
        let mut failures: u32 = 0;

        loop {
            match self.connect().await {
                Ok((inbound, up_tx)) => {
                    info!("agent stream open — waiting for jobs");
                    let started = Instant::now();
                    let outcome = self.serve(inbound, up_tx).await;
                    let uptime = started.elapsed();

                    if let Some(status) = outcome {
                        error!(code = ?status.code(), "control plane rejected the agent (terminal); not retrying");
                        return Err(AgentError::Status(status));
                    }
                    if uptime >= Duration::from_secs(MIN_UPTIME_FOR_RESET_SECS) {
                        failures = 0;
                        info!("agent stream closed; reconnecting");
                    } else {
                        failures += 1;
                        warn!(
                            uptime_ms = u64::try_from(uptime.as_millis()).unwrap_or(u64::MAX),
                            attempt = failures,
                            max,
                            "stream closed immediately after connect; backing off"
                        );
                        if max != 0 && failures >= max {
                            error!("giving up after {failures} short-lived connections");
                            return Err(AgentError::StreamClosed);
                        }
                    }
                }
                Err(e) => {
                    if is_terminal(&e) {
                        error!(error = %e, "terminal error connecting (bad/revoked credentials?); not retrying");
                        return Err(e);
                    }
                    failures += 1;
                    warn!(error = %e, attempt = failures, max, "failed to connect to control plane");
                    if max != 0 && failures >= max {
                        error!("giving up after {failures} connection attempts");
                        return Err(e);
                    }
                }
            }
            tokio::time::sleep(backoff_delay(base, failures)).await;
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
                app_id: Some(scylla_protocol::services::common::AppId {
                    value: self.config.app_id.clone(),
                }),
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

    /// Run dispatched jobs until the stream ends. Returns `Some(status)` when the
    /// stream ended with a **terminal** gRPC code (the caller must stop, not
    /// retry); `None` on a clean close or a transient error (caller reconnects).
    async fn serve(
        &self,
        mut inbound: Streaming<AgentDown>,
        up_tx: mpsc::Sender<AgentUp>,
    ) -> Option<tonic::Status> {
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
                Ok(None) => return None,
                Err(status) => {
                    warn!(error = %status, code = ?status.code(), "agent stream error");
                    return is_terminal_code(status.code()).then_some(status);
                }
            }
        }
    }
}

/// Seconds an agent stream must stay up before we treat the connection as
/// healthy and reset the reconnect-failure counter.
const MIN_UPTIME_FOR_RESET_SECS: u64 = 5;
/// Upper bound on the exponential reconnect backoff.
const MAX_BACKOFF_SECS: u64 = 60;

/// Exponential backoff capped at [`MAX_BACKOFF_SECS`]. `failures == 0` (a healthy
/// reconnect) waits the base interval; each further consecutive failure doubles
/// it, up to 64× the base or the cap, whichever is smaller.
fn backoff_delay(base_secs: u64, failures: u32) -> Duration {
    let shift = failures.saturating_sub(1).min(6);
    let secs = base_secs
        .saturating_mul(1u64 << shift)
        .clamp(1, MAX_BACKOFF_SECS);
    Duration::from_secs(secs)
}

/// gRPC codes that mean "this will never succeed as-is" — a revoked/disabled
/// secret, an inactive app, or a deleted agent. Retrying only adds load.
fn is_terminal_code(code: Code) -> bool {
    matches!(
        code,
        Code::Unauthenticated | Code::PermissionDenied | Code::NotFound
    )
}

fn is_terminal(err: &AgentError) -> bool {
    matches!(err, AgentError::Status(s) if is_terminal_code(s.code()))
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
