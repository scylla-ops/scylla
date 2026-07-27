use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Streaming;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Code, Request};
use tracing::{error, info, warn};

use scylla_core::domain::entities::PipelineNode;
use scylla_core::domain::value_objects::pipeline::{
    EnvKey, EnvVar, NodeId, Shell, Step, WorkingDir,
};
use scylla_protocol::agent::v1::agent_service_client::AgentServiceClient;
use scylla_protocol::agent::v1::{AgentDown, AgentNode, AgentUp, agent_down, agent_node};
use scylla_protocol::app::v1::IssueTokenRequest;
use scylla_protocol::app::v1::app_auth_service_client::AppAuthServiceClient;
use scylla_protocol::common::v1 as common;
use scylla_protocol::exec::v1 as exec;

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::executor::Executor;
use crate::reporter::StatusPublisher;
use scylla_core::application::JobEvent;

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
        let url = self.config.control_plane_url.clone();
        let mut endpoint =
            Channel::from_shared(url.clone()).map_err(|e| AgentError::InvalidUrl {
                url: url.clone(),
                message: e.to_string(),
            })?;
        // Keepalive so a half-open connection (proxy/NAT idle drop, or a control
        // plane that vanished without a FIN) is detected instead of hanging the
        // agent forever: pings while idle, and a missed pong tears the stream
        // down so the outer reconnect loop kicks in.
        endpoint = endpoint
            .http2_keep_alive_interval(Duration::from_secs(20))
            .keep_alive_timeout(Duration::from_secs(10))
            .keep_alive_while_idle(true)
            .tcp_keepalive(Some(Duration::from_secs(30)));
        // An https:// control plane terminates TLS at the proxy (which then
        // speaks h2c to the backend); load the host's native roots so the
        // handshake succeeds. Plain http:// (local dev) stays cleartext h2c.
        if url.starts_with("https://") {
            endpoint = endpoint.tls_config(ClientTlsConfig::new().with_native_roots())?;
        }
        let channel = endpoint.connect().await?;

        let token = AppAuthServiceClient::new(channel.clone())
            .issue_token(IssueTokenRequest {
                app_id: Some(common::AppId {
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
                        let job_id = dispatch.job_id.unwrap_or_default().value;
                        let pipeline_id = dispatch.pipeline_id.unwrap_or_default().value;
                        info!(
                            %job_id,
                            %pipeline_id,
                            nodes = dispatch.nodes.len(),
                            "received job"
                        );
                        // Collect secret-sourced values to scrub from logs before
                        // the nodes are consumed by `to_domain_nodes`.
                        let masked_values: Vec<String> = dispatch
                            .nodes
                            .iter()
                            .flat_map(|n| &n.env)
                            .filter(|e| e.masked && !e.value.is_empty())
                            .map(|e| e.value.clone())
                            .collect();
                        let nodes = match to_domain_nodes(dispatch.nodes) {
                            Ok(nodes) => nodes,
                            Err(e) => {
                                // Silently skipping would strand the job in
                                // `pending` forever (it is already assigned to
                                // this agent) — fail it upstream instead.
                                warn!(%job_id, error = %e, "invalid dispatch nodes, failing job");
                                let publisher = StatusPublisher::new(up_tx.clone(), job_id.clone());
                                if let Err(pe) = publisher.emit(JobEvent::JobStarted).await {
                                    warn!(%job_id, error = %pe, "failed to report job start");
                                } else if let Err(pe) = publisher
                                    .emit(JobEvent::JobFailed {
                                        error: format!("invalid dispatch: {e}"),
                                    })
                                    .await
                                {
                                    warn!(%job_id, error = %pe, "failed to report job failure");
                                }
                                continue;
                            }
                        };
                        let executor = Executor::new(
                            up_tx.clone(),
                            job_id.clone(),
                            self.config.workspace_root.clone(),
                            self.config.keep_workspace,
                            masked_values,
                        );
                        // V1: sequential — finish the job before accepting the next.
                        if let Err(e) = executor.run(nodes).await {
                            // The JobReporter inside run() already pushed the
                            // terminal JobFailed upstream — this log is local
                            // operator context only.
                            error!(%job_id, error = %e, "job execution failed");
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
            let id =
                NodeId::new(&n.node_id.unwrap_or_default().value).map_err(|e| e.to_string())?;
            let deps = n
                .deps
                .iter()
                .map(|d| NodeId::new(&d.value).map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            let working_dir = match n.working_dir.trim() {
                "" => None,
                s => Some(WorkingDir::new(s).map_err(|e| e.to_string())?),
            };
            let env = n
                .env
                .into_iter()
                .map(|e| {
                    // The agent only ever receives already-resolved literals
                    // (secret refs are resolved control-plane-side at dispatch).
                    let key = EnvKey::new(&e.key).map_err(|err| err.to_string())?;
                    Ok::<_, String>(EnvVar::literal(key, e.value))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let step = match n.step {
                Some(agent_node::Step::Exec(e)) => {
                    Step::exec(e.command, e.args).map_err(|err| err.to_string())?
                }
                Some(agent_node::Step::Script(s)) => {
                    Step::script(s.script, shell_from_proto(s.shell))
                        .map_err(|err| err.to_string())?
                }
                None => return Err("dispatch node is missing its step".to_string()),
            };
            Ok(PipelineNode::new(id, deps, step, working_dir, env))
        })
        .collect()
}

fn shell_from_proto(raw: i32) -> Shell {
    match exec::Shell::try_from(raw).unwrap_or_default() {
        exec::Shell::Bash => Shell::Bash,
        exec::Shell::Sh | exec::Shell::Unspecified => Shell::Sh,
    }
}
