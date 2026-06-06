//! Pipeline DAG executor.
//!
//! Walks the DAG in topological order, running independent nodes in parallel.
//! On first node failure:
//!   * cancels in-flight sibling tasks via [`CancellationToken`];
//!   * emits `NodeSkipped` for every node that hasn't reached a terminal state;
//!   * emits a single `JobFailed` on scope exit (via [`JobReporter`]).
//!
//! Status and log events are sent as `AgentUp` messages on the agent stream.

use chrono::Utc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use scylla_core::application::JobEvent;
use scylla_core::domain::entities::PipelineNode;
use scylla_core::domain::value_objects::job::LogStream;
use scylla_protocol::services::agent::{AgentUp, JobLogLine, agent_up};
use scylla_protocol::services::common;

use crate::error::ExecutionError;
use crate::plan::DagPlan;
use crate::reporter::{JobReporter, StatusPublisher};

/// Executes a pipeline DAG by walking nodes in topological order, spawning
/// parallel tasks for independent nodes.
pub struct Executor {
    up_tx: mpsc::Sender<AgentUp>,
    job_id: String,
}

impl Executor {
    pub fn new(up_tx: mpsc::Sender<AgentUp>, job_id: String) -> Self {
        Self { up_tx, job_id }
    }

    /// Execute all nodes in the DAG, respecting dependencies.
    pub async fn run(&self, nodes: Vec<PipelineNode>) -> Result<(), ExecutionError> {
        let publisher = StatusPublisher::new(self.up_tx.clone(), self.job_id.clone());
        let mut reporter = JobReporter::start(publisher.clone()).await?;
        let cancel = CancellationToken::new();

        let outcome = self.execute(&nodes, &publisher, &cancel).await;

        match &outcome {
            Ok(()) => reporter.commit_success(),
            Err(err) => reporter.commit_failure(format!("job failed: {err}")),
        }
        reporter.finalize().await?;
        outcome
    }

    async fn execute(
        &self,
        nodes: &[PipelineNode],
        publisher: &StatusPublisher,
        cancel: &CancellationToken,
    ) -> Result<(), ExecutionError> {
        let mut plan = DagPlan::build(nodes);

        while !plan.is_exhausted() {
            let batch = plan.drain_ready();
            if batch.is_empty() {
                // Pending nodes remain but none are ready: cycle or dangling deps.
                skip_all_pending(&mut plan, publisher).await?;
                return Err(ExecutionError::DanglingDeps);
            }

            let mut running = self
                .dispatch_batch(&batch, &plan, publisher, cancel)
                .await?;

            while let Some(joined) = running.join_next().await {
                let (id, result) = joined.map_err(|e| ExecutionError::NodeTaskPanic {
                    message: e.to_string(),
                })?;

                match result {
                    Ok(()) => {
                        info!(node_id = %id, "node completed");
                        publisher
                            .emit(JobEvent::NodeCompleted {
                                node_id: id.clone(),
                            })
                            .await?;
                        plan.mark_completed(&id);
                    }
                    Err(err) => {
                        error!(node_id = %id, error = %err, "node failed");
                        publisher
                            .emit(JobEvent::NodeFailed {
                                node_id: id.clone(),
                                error: err.to_string(),
                            })
                            .await?;
                        plan.mark_terminal(&id);

                        cancel.cancel();
                        drain_cancelled(&mut running, &mut plan, publisher).await?;
                        skip_all_pending(&mut plan, publisher).await?;
                        return Err(err);
                    }
                }
            }
        }

        Ok(())
    }

    async fn dispatch_batch(
        &self,
        batch: &[&str],
        plan: &DagPlan<'_>,
        publisher: &StatusPublisher,
        cancel: &CancellationToken,
    ) -> Result<JoinSet<(String, Result<(), ExecutionError>)>, ExecutionError> {
        let mut running: JoinSet<(String, Result<(), ExecutionError>)> = JoinSet::new();

        for &node_id in batch {
            publisher
                .emit(JobEvent::NodeStarted {
                    node_id: node_id.to_string(),
                })
                .await?;

            let spec = plan.lookup(node_id).clone();
            let tx = self.up_tx.clone();
            let job_id = self.job_id.clone();
            let token = cancel.clone();
            let id = node_id.to_string();

            running.spawn(async move {
                let result = run_node(&id, &spec, &tx, &job_id, token).await;
                (id, result)
            });
        }

        Ok(running)
    }
}

/// Drain the in-flight JoinSet after cancellation: each survivor is reported as
/// [`JobEvent::NodeSkipped`] regardless of how its child exited.
async fn drain_cancelled(
    running: &mut JoinSet<(String, Result<(), ExecutionError>)>,
    plan: &mut DagPlan<'_>,
    publisher: &StatusPublisher,
) -> Result<(), ExecutionError> {
    while let Some(joined) = running.join_next().await {
        let (id, _result) = joined.map_err(|e| ExecutionError::NodeTaskPanic {
            message: e.to_string(),
        })?;
        publisher
            .emit(JobEvent::NodeSkipped {
                node_id: id.clone(),
            })
            .await?;
        plan.mark_terminal(&id);
    }
    Ok(())
}

/// Emit [`JobEvent::NodeSkipped`] for every node still pending in the plan.
async fn skip_all_pending(
    plan: &mut DagPlan<'_>,
    publisher: &StatusPublisher,
) -> Result<(), ExecutionError> {
    let remaining: Vec<String> = plan.pending().map(str::to_string).collect();
    for id in remaining {
        publisher
            .emit(JobEvent::NodeSkipped {
                node_id: id.clone(),
            })
            .await?;
        plan.mark_terminal(&id);
    }
    Ok(())
}

/// Run a single node's command, streaming stdout/stderr to the agent stream.
///
/// Listens to `cancel` in parallel with `child.wait()`: if cancellation fires
/// first, the child is killed and [`ExecutionError::Cancelled`] is returned.
async fn run_node(
    node_id: &str,
    spec: &PipelineNode,
    up_tx: &mpsc::Sender<AgentUp>,
    job_id: &str,
    cancel: CancellationToken,
) -> Result<(), ExecutionError> {
    let mut child = match Command::new(spec.command())
        .args(spec.args())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            // Surface spawn failure (e.g. command-not-found) on the stderr log
            // stream so it shows up in the node logs view, not just in the
            // NodeFailed status event.
            publish_log_line(
                up_tx,
                job_id,
                node_id,
                LogStream::Stderr,
                format!("failed to spawn `{}`: {e}", spec.command()),
            )
            .await;
            return Err(ExecutionError::Spawn(e));
        }
    };
    // INVARIANT: stdout/stderr were configured as Stdio::piped() on the Command above.
    let stdout_handle = spawn_log_streamer(
        child.stdout.take().expect("stdout was piped"),
        LogStream::Stdout,
        node_id.to_string(),
        job_id.to_string(),
        up_tx.clone(),
    );
    let stderr_handle = spawn_log_streamer(
        child.stderr.take().expect("stderr was piped"),
        LogStream::Stderr,
        node_id.to_string(),
        job_id.to_string(),
        up_tx.clone(),
    );

    let wait_outcome: Result<(), ExecutionError> = tokio::select! {
        biased;
        () = cancel.cancelled() => {
            let _ = child.start_kill();
            // still wait so the child is reaped and log streams flush
            let _ = child.wait().await;
            Err(ExecutionError::Cancelled { node_id: node_id.to_string() })
        }
        status = child.wait() => {
            exit_status_to_result(node_id, status)
        }
    };

    // Ensure log streams flush before returning.
    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

    wait_outcome
}

fn exit_status_to_result(
    node_id: &str,
    status: std::io::Result<std::process::ExitStatus>,
) -> Result<(), ExecutionError> {
    let status = status.map_err(ExecutionError::Spawn)?;
    if status.success() {
        Ok(())
    } else {
        Err(match status.code() {
            Some(code) => ExecutionError::NodeFailed {
                node_id: node_id.to_string(),
                exit_code: code,
            },
            None => ExecutionError::NodeKilled {
                node_id: node_id.to_string(),
            },
        })
    }
}

fn spawn_log_streamer<R>(
    reader: R,
    stream: LogStream,
    node_id: String,
    job_id: String,
    up_tx: mpsc::Sender<AgentUp>,
) -> tokio::task::JoinHandle<()>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if !publish_log_line(&up_tx, &job_id, &node_id, stream, line).await {
                break;
            }
        }
    })
}

/// Send a single log line as a `AgentUp`. Returns `false` if the channel is
/// closed (caller should stop emitting).
async fn publish_log_line(
    up_tx: &mpsc::Sender<AgentUp>,
    job_id: &str,
    node_id: &str,
    stream: LogStream,
    line: String,
) -> bool {
    let log = JobLogLine {
        job_id: Some(common::JobId {
            value: job_id.to_string(),
        }),
        node_id: Some(common::NodeId {
            value: node_id.to_string(),
        }),
        stream: stream.as_str().to_string(),
        line,
        timestamp: Utc::now().to_rfc3339(),
    };
    if up_tx
        .send(AgentUp {
            payload: Some(agent_up::Payload::Log(log)),
        })
        .await
        .is_err()
    {
        warn!("agent up-stream channel closed");
        return false;
    }
    true
}
