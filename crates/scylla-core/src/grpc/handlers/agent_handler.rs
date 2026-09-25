use crate::application::actions::app_only;
use crate::application::agent::{RecordAgentHost, TouchAgent};
use crate::application::job::{AppendJobLog, RecordJobStatus};
use crate::application::{AgentDispatch, AgentUseCases, JobLogUseCases, JobUseCases};
use crate::application::{JobDispatch, JobEvent};
use crate::domain::agent::AgentHost;
use crate::domain::caller::CallerContext;
use crate::domain::clock;
use crate::domain::ids::{AppId, JobId};
use crate::domain::job::JobLog;
use crate::domain::pipeline::{NodeId, Step};
use crate::extract_auth_context;
use crate::grpc::convert::dt;
use crate::grpc::mappers::domain_error_to_status;
use crate::infrastructure::{InMemoryAgentRegistry, InMemoryJobLogStream};
use derive_more::Constructor;
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
pub struct AgentHandler {
    actions: Arc<Actions>,
    jobs: Arc<JobUseCases>,
    logs: Arc<JobLogUseCases>,
    agents: Arc<AgentUseCases>,
    registry: Arc<InMemoryAgentRegistry>,
    log_stream: Arc<InMemoryJobLogStream>,
    pending_signal: Arc<Notify>,
}

#[async_trait::async_trait]
impl AgentService for AgentHandler {
    type OpenStream = Pin<Box<dyn Stream<Item = Result<AgentDown, Status>> + Send + 'static>>;

    async fn open(
        &self,
        request: Request<Streaming<AgentUp>>,
    ) -> Result<Response<Self::OpenStream>, Status> {
        let caller = caller!(request);
        self.actions
            .run(&*self.agents, &caller, TouchAgent)
            .await
            .map_err(domain_error_to_status)?;
        let app_id = app_only(&caller).map_err(domain_error_to_status)?;

        let inbound = request.into_inner();
        let (conn_id, dispatch_rx) = self.registry.register(&app_id);
        self.pending_signal.notify_one();

        let reader = Reader {
            caller,
            app_id: app_id.clone(),
            actions: self.actions.clone(),
            jobs: self.jobs.clone(),
            logs: self.logs.clone(),
            agents: self.agents.clone(),
            log_stream: self.log_stream.clone(),
            registry: self.registry.clone(),
        };
        tokio::spawn(reader.read(inbound, conn_id));

        // The stream owns a `DisconnectGuard`: a half-closed client must not leave a stale sender.
        let down = ReceiverStream::new(dispatch_rx).map(|d| Ok(dispatch_to_proto(&d)));
        let guarded = GuardedStream {
            inner: down,
            _guard: DisconnectGuard {
                registry: self.registry.clone(),
                app_id,
                conn_id,
            },
        };
        Ok(Response::new(Box::pin(guarded)))
    }
}

struct Reader {
    caller: CallerContext,
    app_id: AppId,
    actions: Arc<Actions>,
    jobs: Arc<JobUseCases>,
    logs: Arc<JobLogUseCases>,
    agents: Arc<AgentUseCases>,
    log_stream: Arc<InMemoryJobLogStream>,
    registry: Arc<InMemoryAgentRegistry>,
}

impl Reader {
    async fn read(self, mut inbound: Streaming<AgentUp>, conn_id: u64) {
        while let Ok(Some(up)) = inbound.message().await {
            match up.payload {
                Some(agent_up::Payload::Status(status)) => {
                    let job_id = JobId::new(status.job_id.clone().unwrap_or_default().value);
                    if let Some(event) = scylla_proto::convert::status_to_job_event(&status) {
                        self.record_status(job_id, event).await;
                    }
                    self.touch().await;
                }
                Some(agent_up::Payload::Log(line)) => {
                    if let Some(log) = log_line_to_domain(&line) {
                        self.append_log(log).await;
                    }
                }
                Some(agent_up::Payload::Hello(hello)) => {
                    let host = RecordAgentHost {
                        host: hello_to_domain(&hello),
                    };
                    if let Err(e) = self.actions.run(&*self.agents, &self.caller, host).await {
                        warn!(app_id = %self.app_id, error = %e, "failed to record agent host");
                    }
                    self.touch().await;
                }
                None => {}
            }
        }
        // Only if this connection is still the live one: a reconnect may have replaced it.
        self.touch().await;
        self.registry.unregister_if_current(&self.app_id, conn_id);
    }

    async fn touch(&self) {
        if let Err(e) = self
            .actions
            .run(&*self.agents, &self.caller, TouchAgent)
            .await
        {
            warn!(app_id = %self.app_id, error = %e, "failed to update agent last_seen");
        }
    }

    async fn record_status(&self, job_id: JobId, event: JobEvent) {
        let record = RecordJobStatus {
            job_id: job_id.clone(),
            event: event.clone(),
        };
        let recorded = self.actions.run(&*self.jobs, &self.caller, record).await;
        if let Err(e) = &recorded {
            warn!(app_id = %self.app_id, job_id = %job_id, error = %e, "failed to record job status");
        }
        // Open at start so a reader tailing before the first line joins; subscribe never creates a channel.
        // A terminal report frees the slot even if the write failed: the agent no longer runs the job.
        match event {
            JobEvent::JobStarted if recorded.is_ok() => self.log_stream.open(job_id.as_str()),
            JobEvent::JobCompleted | JobEvent::JobFailed { .. } => {
                self.log_stream.close(job_id.as_str());
                self.registry.release(&self.app_id);
            }
            _ => {}
        }
    }

    async fn append_log(&self, log: JobLog) {
        let append = AppendJobLog { log: log.clone() };
        match self.actions.run(&*self.logs, &self.caller, append).await {
            Ok(_) => self.log_stream.publish(log),
            Err(e) => {
                warn!(app_id = %self.app_id, job_id = %log.job_id(), error = %e, "failed to append job log");
            }
        }
    }
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
        reported_at: clock::now(),
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
    let timestamp = dt(line.timestamp).unwrap_or_else(clock::now);
    Some(JobLog::new(
        JobId::new(line.job_id.clone().unwrap_or_default().value),
        node_id,
        stream,
        line.line.clone(),
        timestamp,
    ))
}
