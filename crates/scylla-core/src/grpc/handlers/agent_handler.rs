use crate::application::job::{AppendJobLog, RecordJobStatus};
use crate::application::{
    AgentDispatch, AgentRepository, JobLogRepository, JobLogUseCases, JobRepository, JobUseCases,
};
use crate::application::{JobDispatch, JobEvent};
use crate::domain::caller::CallerContext;
use crate::extract_auth_context;
use crate::grpc::convert::dt;
use crate::infrastructure::{InMemoryAgentRegistry, InMemoryJobLogStream};
use derive_more::Constructor;
use scylla_domain::domain::agent::AgentHost;
use scylla_domain::domain::ids::{AppId, JobId};
use scylla_domain::domain::job::JobLog;
use scylla_domain::domain::pipeline::{NodeId, Step};
use scylla_extension::Actions;
use scylla_proto::agent::v1::{
    AgentDown, AgentNode, AgentUp, JobDispatch as ProtoJobDispatch, ResolvedEnv, agent_down,
    agent_node, agent_service_server::AgentService, agent_up,
};
use scylla_proto::common::v1 as common;
use scylla_proto::exec::v1 as exec;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Notify;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::warn;

#[derive(Constructor)]
pub struct AgentHandler<J, L>
where
    J: JobRepository,
    L: JobLogRepository,
{
    registry: Arc<InMemoryAgentRegistry>,
    log_stream: Arc<InMemoryJobLogStream>,
    actions: Arc<Actions>,
    job_use_cases: Arc<JobUseCases<J>>,
    log_use_cases: Arc<JobLogUseCases<L, InMemoryJobLogStream>>,
    agent_repo: Arc<dyn AgentRepository>,
    pending_signal: Arc<Notify>,
}

#[async_trait::async_trait]
impl<J: JobRepository + Send + Sync + 'static, L: JobLogRepository + Send + Sync + 'static>
    AgentService for AgentHandler<J, L>
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

        self.pending_signal.notify_one();

        tokio::spawn(read_reports(
            inbound,
            app_id.clone(),
            self.actions.clone(),
            self.job_use_cases.clone(),
            self.log_use_cases.clone(),
            self.log_stream.clone(),
            self.registry.clone(),
            self.agent_repo.clone(),
            conn_id,
        ));

        // The stream owns a `DisconnectGuard`: a half-closed client must not leave a stale sender.
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
async fn read_reports<J, L>(
    mut inbound: Streaming<AgentUp>,
    app_id: AppId,
    actions: Arc<Actions>,
    job_use_cases: Arc<JobUseCases<J>>,
    log_use_cases: Arc<JobLogUseCases<L, InMemoryJobLogStream>>,
    log_stream: Arc<InMemoryJobLogStream>,
    registry: Arc<InMemoryAgentRegistry>,
    agent_repo: Arc<dyn AgentRepository>,
    conn_id: u64,
) where
    J: JobRepository + Send + Sync + 'static,
    L: JobLogRepository + Send + Sync + 'static,
{
    let caller = CallerContext::App(app_id.clone());
    touch_last_seen(&agent_repo, &app_id).await;
    while let Ok(Some(up)) = inbound.message().await {
        match up.payload {
            Some(agent_up::Payload::Status(status)) => {
                let job_id = JobId::new(status.job_id.clone().unwrap_or_default().value);
                if let Some(event) = scylla_proto::convert::status_to_job_event(&status) {
                    let record = RecordJobStatus {
                        job_id: job_id.clone(),
                        event: event.clone(),
                    };
                    if let Err(e) = actions.run(&*job_use_cases, &caller, record).await {
                        warn!(app_id = %app_id, job_id = %job_id, error = %e, "failed to record job status");
                    }
                    // Open at start so a reader tailing before the first line joins; subscribe never creates a channel.
                    if matches!(event, JobEvent::JobStarted) {
                        log_stream.open(job_id.as_str());
                    }
                    if matches!(event, JobEvent::JobCompleted | JobEvent::JobFailed { .. }) {
                        log_stream.close(job_id.as_str());
                        registry.release(&app_id);
                    }
                }
                touch_last_seen(&agent_repo, &app_id).await;
            }
            Some(agent_up::Payload::Log(line)) => {
                if let Some(log) = log_line_to_domain(&line) {
                    let append = AppendJobLog { log: log.clone() };
                    if let Err(e) = actions.run(&*log_use_cases, &caller, append).await {
                        warn!(app_id = %app_id, job_id = %log.job_id(), error = %e, "failed to append job log");
                    } else {
                        log_stream.publish(log);
                    }
                }
            }
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
    // Only if this connection is still the live one: a reconnect may have replaced it.
    touch_last_seen(&agent_repo, &app_id).await;
    registry.unregister_if_current(&app_id, conn_id);
}

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

fn hello_to_domain(hello: &scylla_proto::agent::v1::AgentHello) -> AgentHost {
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
            shell: scylla_proto::convert::shell_to_proto(*shell) as i32,
        }),
    }
}

fn log_line_to_domain(line: &scylla_proto::agent::v1::JobLogLine) -> Option<JobLog> {
    let node_id_str = line.node_id.clone().unwrap_or_default().value;
    let node_id = NodeId::new(&node_id_str)
        .map_err(|e| warn!(node_id = %node_id_str, error = %e, "invalid node_id in agent log"))
        .ok()?;
    let stream = scylla_proto::convert::log_stream_from_proto(line.stream);
    let timestamp = dt(line.timestamp).unwrap_or_else(chrono::Utc::now);
    Some(JobLog::new(
        JobId::new(line.job_id.clone().unwrap_or_default().value),
        node_id,
        stream,
        line.line.clone(),
        timestamp,
    ))
}
