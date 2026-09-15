use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Streaming;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Code, Request};
use tracing::{error, info, warn};

use scylla_domain::domain::pipeline::PipelineNode;
use scylla_domain::domain::pipeline::{EnvKey, EnvVar, NodeId, Step, WorkingDir};
use scylla_proto::agent::v1::agent_service_client::AgentServiceClient;
use scylla_proto::agent::v1::{AgentDown, AgentNode, AgentUp, agent_down, agent_node, agent_up};
use scylla_proto::app::v1::IssueTokenRequest;
use scylla_proto::app::v1::app_auth_service_client::AppAuthServiceClient;
use scylla_proto::common::v1 as common;

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::executor::Executor;
use crate::reporter::StatusPublisher;
use scylla_domain::JobEvent;

pub struct Agent {
    config: AgentConfig,
}

impl Agent {
    #[must_use]
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// A terminal rejection (revoked secret, disabled app, deleted agent) stops the agent: retrying cannot fix credentials.
    /// The failure counter resets only after a connection that stayed up, so a connect-then-close loop still backs off.
    pub async fn run(&self) -> Result<(), AgentError> {
        let base = self.config.reconnect_backoff_secs;
        let max = self.config.max_reconnect_attempts;
        let mut failures: u32 = 0;

        loop {
            match self.connect().await {
                Ok((inbound, up_tx)) => {
                    info!("agent stream open — waiting for jobs");
                    send_hello(&up_tx).await;
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

    async fn connect(&self) -> Result<(Streaming<AgentDown>, mpsc::Sender<AgentUp>), AgentError> {
        let url = self.config.control_plane_url.clone();
        let mut endpoint =
            Channel::from_shared(url.clone()).map_err(|e| AgentError::InvalidUrl {
                url: url.clone(),
                message: e.to_string(),
            })?;
        // Keepalive: a half-open connection (NAT drop, control plane gone without FIN) must not hang the agent.
        endpoint = endpoint
            .http2_keep_alive_interval(Duration::from_secs(20))
            .keep_alive_timeout(Duration::from_secs(10))
            .keep_alive_while_idle(true)
            .tcp_keepalive(Some(Duration::from_secs(30)));
        // https:// terminates at the proxy; load native roots. http:// stays h2c.
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
                                // Skipping would strand the job in `pending`: it is already assigned to this agent.
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
                        if let Err(e) = executor.run(nodes).await {
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

async fn send_hello(up_tx: &mpsc::Sender<AgentUp>) {
    let hello = crate::host::hello();
    info!(
        version = %hello.version,
        os = %hello.os,
        arch = %hello.arch,
        hostname = %hello.hostname,
        cpus = hello.cpu_count,
        memory_mb = hello.total_memory_mb,
        "reporting agent host to control plane"
    );
    if let Err(e) = up_tx
        .send(AgentUp {
            payload: Some(agent_up::Payload::Hello(hello)),
        })
        .await
    {
        warn!(error = %e, "failed to report agent host; continuing without it");
    }
}

const MIN_UPTIME_FOR_RESET_SECS: u64 = 5;
const MAX_BACKOFF_SECS: u64 = 60;

fn backoff_delay(base_secs: u64, failures: u32) -> Duration {
    let shift = failures.saturating_sub(1).min(6);
    let secs = base_secs
        .saturating_mul(1u64 << shift)
        .clamp(1, MAX_BACKOFF_SECS);
    Duration::from_secs(secs)
}

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
                    let key = EnvKey::new(&e.key).map_err(|err| err.to_string())?;
                    Ok::<_, String>(EnvVar::literal(key, e.value))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let step = match n.step {
                Some(agent_node::Step::Exec(e)) => {
                    Step::exec(e.command, e.args).map_err(|err| err.to_string())?
                }
                Some(agent_node::Step::Script(s)) => {
                    Step::script(s.script, scylla_proto::convert::shell_from_proto(s.shell))
                        .map_err(|err| err.to_string())?
                }
                None => return Err("dispatch node is missing its step".to_string()),
            };
            Ok(PipelineNode::new(id, deps, step, working_dir, env))
        })
        .collect()
}
