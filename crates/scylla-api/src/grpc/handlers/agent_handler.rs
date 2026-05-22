use crate::extract_auth_context;
use derive_more::Constructor;
use scylla_core::application::caller::CallerContext;
use scylla_core::application::{
    JobLogRepository, JobLogUseCases, JobRepository, JobUseCases, PermissionService,
    AgentRepository,
};
use scylla_core::domain::entities::{AppId, JobId, JobLog};
use scylla_core::domain::value_objects::job::{JobEvent, LogStream};
use scylla_core::domain::value_objects::pipeline::{JobDispatch, NodeId};
use scylla_core::infrastructure::{InMemoryJobLogStream, InMemoryAgentRegistry};
use scylla_protocol::services::agent::{
    JobDispatch as ProtoJobDispatch, JobEventKind, AgentDown, AgentNode, AgentUp,
    agent_down, agent_service_server::AgentService, agent_up,
};
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::warn;

/// Persistent agent stream: an authenticated App opens it, receives job
/// dispatches, and streams back status + log events. Presence in the registry
/// is the open stream. Reports are persisted as the App principal, so the
/// agent role's `writeJobStatus` / `writeJobLog` grants gate them via Cedar.
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
        let dispatch_rx = self.registry.register(&app_id);

        // Inbound reports run in the background; the App principal authorizes the
        // persistence (writeJobStatus / writeJobLog) via its agent grant.
        tokio::spawn(read_reports(
            inbound,
            app_id.clone(),
            self.job_use_cases.clone(),
            self.log_use_cases.clone(),
            self.log_stream.clone(),
            self.registry.clone(),
            self.agent_repo.clone(),
        ));

        // Outbound: forward dispatches placed in the registry to the wire.
        let down = ReceiverStream::new(dispatch_rx).map(|d| Ok(dispatch_to_proto(&d)));
        Ok(Response::new(Box::pin(down)))
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
                let job_id = JobId::new(&status.job_id);
                if let Some(event) = status_to_event(&status) {
                    if let Err(e) = job_use_cases.record_status(&caller, &job_id, &event).await {
                        warn!(app_id = %app_id, job_id = %status.job_id, error = %e, "failed to record job status");
                    }
                }
                touch_last_seen(&agent_repo, &app_id).await;
            }
            Some(agent_up::Payload::Log(line)) => {
                if let Some(log) = log_line_to_domain(&line) {
                    if let Err(e) = log_use_cases.append(&caller, &log).await {
                        warn!(app_id = %app_id, job_id = %line.job_id, error = %e, "failed to append job log");
                    } else {
                        log_stream.publish(log);
                    }
                }
            }
            None => {}
        }
    }
    // Stamp final activity on disconnect, then drop the stream from the registry.
    touch_last_seen(&agent_repo, &app_id).await;
    registry.unregister(&app_id);
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
            node_id: n.id().to_string(),
            deps: n.deps().iter().map(ToString::to_string).collect(),
            command: n.command().to_string(),
            args: n.args().to_vec(),
        })
        .collect();
    AgentDown {
        payload: Some(agent_down::Payload::Dispatch(ProtoJobDispatch {
            job_id: dispatch.job_id.clone(),
            pipeline_id: dispatch.pipeline_id.clone(),
            nodes,
        })),
    }
}

fn status_to_event(status: &scylla_protocol::services::agent::JobStatus) -> Option<JobEvent> {
    let node_id = || status.node_id.clone();
    let error = || status.error.clone();
    Some(match status.kind() {
        JobEventKind::JobStarted => JobEvent::JobStarted,
        JobEventKind::NodeStarted => JobEvent::NodeStarted { node_id: node_id() },
        JobEventKind::NodeCompleted => JobEvent::NodeCompleted { node_id: node_id() },
        JobEventKind::NodeFailed => JobEvent::NodeFailed {
            node_id: node_id(),
            error: error(),
        },
        JobEventKind::NodeSkipped => JobEvent::NodeSkipped { node_id: node_id() },
        JobEventKind::JobCompleted => JobEvent::JobCompleted,
        JobEventKind::JobFailed => JobEvent::JobFailed { error: error() },
    })
}

fn log_line_to_domain(line: &scylla_protocol::services::agent::JobLogLine) -> Option<JobLog> {
    let node_id = NodeId::new(&line.node_id)
        .map_err(|e| warn!(node_id = %line.node_id, error = %e, "invalid node_id in agent log"))
        .ok()?;
    let stream = LogStream::new(&line.stream).unwrap_or(LogStream::Stdout);
    let timestamp = chrono::DateTime::parse_from_rfc3339(&line.timestamp)
        .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc));
    Some(JobLog::new(
        JobId::new(&line.job_id),
        node_id,
        stream,
        line.line.clone(),
        timestamp,
    ))
}
