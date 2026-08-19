use crate::application::caller::CallerContext;
use crate::application::{
    AgentDispatch, AgentRepository, JobLogRepository, JobLogUseCases, JobRepository, JobUseCases,
    PermissionService,
};
use crate::application::{JobDispatch, JobEvent};
use crate::extract_auth_context;
use crate::grpc::convert::dt;
use crate::infrastructure::{InMemoryAgentRegistry, InMemoryJobLogStream};
use derive_more::Constructor;
use scylla_core::domain::agent::AgentHost;
use scylla_core::domain::ids::{AppId, JobId};
use scylla_core::domain::job::JobLog;
use scylla_core::domain::pipeline::{NodeId, Step};
use scylla_protocol::agent::v1::{
    AgentDown, AgentNode, AgentUp, JobDispatch as ProtoJobDispatch, ResolvedEnv, agent_down,
    agent_node, agent_service_server::AgentService, agent_up,
};
use scylla_protocol::common::v1 as common;
use scylla_protocol::exec::v1 as exec;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Notify;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::warn;

/// Persistent agent stream: an authenticated App opens it, receives job
/// dispatches, and streams back status + log events. Presence in the registry
/// is the open stream. Reports are persisted as the App principal, so the
/// agent role's `writeJobStatus` / `appendJobLog` grants gate them via Cedar.
#[derive(Constructor)]
pub struct AgentHandler<J, L, PS>
where
    J: JobRepository,
    L: JobLogRepository,
    PS: PermissionService,
{
    registry: Arc<InMemoryAgentRegistry>,
    log_stream: Arc<InMemoryJobLogStream>,
    job_use_cases: Arc<JobUseCases<J, PS>>,
    log_use_cases: Arc<JobLogUseCases<L, PS>>,
    /// Durable agent presence: stamped on connect, each report, and disconnect.
    agent_repo: Arc<dyn AgentRepository>,
    /// Poked when an agent connects so the pending-job scheduler re-dispatches
    /// the backlog onto the freshly-available worker.
    pending_signal: Arc<Notify>,
}

#[async_trait::async_trait]
impl<
    J: JobRepository + Send + Sync + 'static,
    L: JobLogRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> AgentService for AgentHandler<J, L, PS>
{
    type OpenStream = Pin<Box<dyn Stream<Item = Result<AgentDown, Status>> + Send + 'static>>;

    async fn open(
        &self,
        request: Request<Streaming<AgentUp>>,
    ) -> Result<Response<Self::OpenStream>, Status> {
        let caller = caller!(request);
        let CallerContext::App(app_id) = caller else {
            return Err(Status::permission_denied(
                "the agent stream requires an app token",
            ));
        };

        let inbound = request.into_inner();
        let (conn_id, dispatch_rx) = self.registry.register(&app_id);

        // A new worker is available — nudge the scheduler to (re)dispatch any
        // jobs left pending because nothing was connected when they were created.
        self.pending_signal.notify_one();

        // Inbound reports run in the background; the App principal authorizes the
        // persistence (writeJobStatus / appendJobLog) via its agent grant.
        tokio::spawn(read_reports(
            inbound,
            app_id.clone(),
            self.job_use_cases.clone(),
            self.log_use_cases.clone(),
            self.log_stream.clone(),
            self.registry.clone(),
            self.agent_repo.clone(),
            conn_id,
        ));

        // Outbound: forward dispatches placed in the registry to the wire. The
        // stream owns a `DisconnectGuard`, so when tonic drops it (client RST /
        // transport close) the registry entry is removed at once — cleanup no
        // longer depends solely on the inbound half closing (a half-closed client
        // would otherwise leave a stale sender that fills the dispatch queue).
        let down = ReceiverStream::new(dispatch_rx).map(|d| Ok(dispatch_to_proto(&d)));
        let guarded = GuardedStream {
            inner: down,
            _guard: DisconnectGuard {
                registry: self.registry.clone(),
                app_id: app_id.clone(),
                conn_id,
            },
        };
        Ok(Response::new(Box::pin(guarded)))
    }
}

#[allow(clippy::too_many_arguments)]
async fn read_reports<J, L, PS>(
    mut inbound: Streaming<AgentUp>,
    app_id: AppId,
    job_use_cases: Arc<JobUseCases<J, PS>>,
    log_use_cases: Arc<JobLogUseCases<L, PS>>,
    log_stream: Arc<InMemoryJobLogStream>,
    registry: Arc<InMemoryAgentRegistry>,
    agent_repo: Arc<dyn AgentRepository>,
    conn_id: u64,
) where
    J: JobRepository + Send + Sync + 'static,
    L: JobLogRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
{
    let caller = CallerContext::App(app_id.clone());
    // Stamp presence on connect (best-effort, never fails the stream).
    touch_last_seen(&agent_repo, &app_id).await;
    // Loop ends when the agent disconnects (Ok(None)) or the stream errors.
    while let Ok(Some(up)) = inbound.message().await {
        match up.payload {
            Some(agent_up::Payload::Status(status)) => {
                let job_id = JobId::new(status.job_id.clone().unwrap_or_default().value);
                if let Some(event) = scylla_protocol::convert::status_to_job_event(&status) {
                    if let Err(e) = job_use_cases.record_status(&caller, &job_id, &event).await {
                        warn!(app_id = %app_id, job_id = %job_id, error = %e, "failed to record job status");
                    }
                    // Open the live channel at job start so a reader tailing a
                    // running job that hasn't logged yet still joins the stream
                    // (subscribe never creates a channel, to avoid leaking one
                    // for an already-finished job).
                    if matches!(event, JobEvent::JobStarted) {
                        log_stream.open(job_id.as_str());
                    }
                    // A terminal job won't emit more log lines — evict its live
                    // channel so the per-job stream map can't grow without bound,
                    // and free the agent's load slot so least-loaded dispatch can
                    // hand it the next job.
                    if matches!(event, JobEvent::JobCompleted | JobEvent::JobFailed { .. }) {
                        log_stream.close(job_id.as_str());
                        registry.release(&app_id);
                    }
                }
                touch_last_seen(&agent_repo, &app_id).await;
            }
            Some(agent_up::Payload::Log(line)) => {
                if let Some(log) = log_line_to_domain(&line) {
                    if let Err(e) = log_use_cases.append(&caller, &log).await {
                        warn!(app_id = %app_id, job_id = %log.job_id(), error = %e, "failed to append job log");
                    } else {
                        log_stream.publish(log);
                    }
                }
            }
            // The agent introducing itself. Introspection only — nothing
            // downstream reads it, so a failed write is logged and swallowed
            // exactly like `touch_last_seen`.
            Some(agent_up::Payload::Hello(hello)) => {
                let host = hello_to_domain(&hello);
                if let Err(e) = agent_repo.record_host(&app_id, &host).await {
                    warn!(app_id = %app_id, error = %e, "failed to record agent host");
                }
                touch_last_seen(&agent_repo, &app_id).await;
            }
            None => {}
        }
    }
    // Stamp final activity on disconnect, then drop the stream from the registry
    // — but only if this connection is still the live one (a reconnect may have
    // replaced it). The outbound `DisconnectGuard` does the same on its side.
    touch_last_seen(&agent_repo, &app_id).await;
    registry.unregister_if_current(&app_id, conn_id);
}

/// RAII cleanup tied to the outbound dispatch stream. When tonic drops the
/// response stream (client disconnect / transport close), this removes the
/// App's registry entry — generation-checked so it never evicts a newer
/// reconnect.
struct DisconnectGuard {
    registry: Arc<InMemoryAgentRegistry>,
    app_id: AppId,
    conn_id: u64,
}

impl Drop for DisconnectGuard {
    fn drop(&mut self) {
        self.registry
            .unregister_if_current(&self.app_id, self.conn_id);
    }
}

/// A stream that owns a value dropped with it. Used to attach a [`DisconnectGuard`]
/// to the agent's outbound stream so client disconnect triggers registry cleanup.
struct GuardedStream<S> {
    inner: S,
    _guard: DisconnectGuard,
}

impl<S: Stream + Unpin> Stream for GuardedStream<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Map the agent's self-description onto the domain. Stored verbatim — the
/// control plane makes no decision on it, so there is nothing to validate. A
/// `0` from the wire means "the agent could not read it", which is `None` here
/// rather than a machine with no CPU or no memory.
fn hello_to_domain(hello: &scylla_protocol::agent::v1::AgentHello) -> AgentHost {
    AgentHost {
        version: hello.version.clone(),
        os: hello.os.clone(),
        arch: hello.arch.clone(),
        hostname: hello.hostname.clone(),
        cpu_count: (hello.cpu_count > 0).then_some(hello.cpu_count),
        total_memory_mb: (hello.total_memory_mb > 0).then_some(hello.total_memory_mb),
        reported_at: chrono::Utc::now(),
    }
}

/// Best-effort durable presence update. Agent introspection must never break
/// the live stream, so failures are logged and swallowed.
async fn touch_last_seen(agent_repo: &Arc<dyn AgentRepository>, app_id: &AppId) {
    if let Err(e) = agent_repo.touch_last_seen(app_id, chrono::Utc::now()).await {
        warn!(app_id = %app_id, error = %e, "failed to update agent last_seen");
    }
}

fn dispatch_to_proto(dispatch: &JobDispatch) -> AgentDown {
    let nodes = dispatch
        .nodes
        .iter()
        .map(|n| AgentNode {
            node_id: Some(common::NodeId {
                value: n.id.clone(),
            }),
            deps: n
                .deps
                .iter()
                .map(|d| common::NodeId { value: d.clone() })
                .collect(),
            working_dir: n.working_dir.clone().unwrap_or_default(),
            // Env is already resolved (secret refs decrypted) with `masked` set
            // for secret-sourced values so the agent can scrub them from logs.
            env: n
                .env
                .iter()
                .map(|ev| ResolvedEnv {
                    key: ev.key.clone(),
                    value: ev.value.clone(),
                    masked: ev.masked,
                })
                .collect(),
            step: Some(step_to_proto(&n.step)),
        })
        .collect();
    AgentDown {
        payload: Some(agent_down::Payload::Dispatch(ProtoJobDispatch {
            job_id: Some(common::JobId {
                value: dispatch.job_id.clone(),
            }),
            pipeline_id: Some(common::PipelineId {
                value: dispatch.pipeline_id.clone(),
            }),
            nodes,
        })),
    }
}

fn step_to_proto(step: &Step) -> agent_node::Step {
    match step {
        Step::Exec { command, args } => agent_node::Step::Exec(exec::ExecStep {
            command: command.clone(),
            args: args.clone(),
        }),
        Step::Script { script, shell } => agent_node::Step::Script(exec::ScriptStep {
            script: script.clone(),
            shell: scylla_protocol::convert::shell_to_proto(*shell) as i32,
        }),
    }
}

fn log_line_to_domain(line: &scylla_protocol::agent::v1::JobLogLine) -> Option<JobLog> {
    let node_id_str = line.node_id.clone().unwrap_or_default().value;
    let node_id = NodeId::new(&node_id_str)
        .map_err(|e| warn!(node_id = %node_id_str, error = %e, "invalid node_id in agent log"))
        .ok()?;
    // Total: unspecified and unknown both fold to stdout, so there is nothing to
    // fail on. Shared with the agent, which encodes the same enum on the way out.
    let stream = scylla_protocol::convert::log_stream_from_proto(line.stream);
    // An agent that omits the timestamp gets server-side now, as before.
    let timestamp = dt(line.timestamp).unwrap_or_else(chrono::Utc::now);
    Some(JobLog::new(
        JobId::new(line.job_id.clone().unwrap_or_default().value),
        node_id,
        stream,
        line.line.clone(),
        timestamp,
    ))
}
