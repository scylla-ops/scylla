//! Pipeline DAG executor.
//!
//! Walks the DAG in topological order, running independent nodes in parallel.
//! On first node failure:
//!   * cancels in-flight sibling tasks via [`CancellationToken`];
//!   * emits `NodeSkipped` for every node that hasn't reached a terminal state;
//!   * emits a single `JobFailed` on scope exit (via [`JobReporter`]).
//!
//! Status and log events are sent as `AgentUp` messages on the agent stream.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use scylla_core::JobEvent;
use scylla_core::domain::entities::PipelineNode;
use scylla_core::domain::value_objects::job::LogStream;
use scylla_core::domain::value_objects::pipeline::{Shell, Step};
use scylla_protocol::agent::v1::{AgentUp, JobLogLine, agent_up};
use scylla_protocol::common::v1 as common;

use crate::error::ExecutionError;
use crate::plan::DagPlan;
use crate::reporter::{JobReporter, StatusPublisher};

/// Executes a pipeline DAG by walking nodes in topological order, spawning
/// parallel tasks for independent nodes.
pub struct Executor {
    up_tx: mpsc::Sender<AgentUp>,
    job_id: String,
    /// Root under which each job gets its own workspace directory.
    workspace_root: PathBuf,
    /// When true, the per-job workspace is left on disk after the run (debug).
    keep_workspace: bool,
    /// Secret-sourced values to redact from every log line before it is emitted.
    masked_values: Arc<Vec<String>>,
}

impl Executor {
    pub fn new(
        up_tx: mpsc::Sender<AgentUp>,
        job_id: String,
        workspace_root: PathBuf,
        keep_workspace: bool,
        masked_values: Vec<String>,
    ) -> Self {
        Self {
            up_tx,
            job_id,
            workspace_root,
            keep_workspace,
            masked_values: Arc::new(masked_values),
        }
    }

    /// Execute all nodes in the DAG, respecting dependencies.
    ///
    /// Allocates one workspace directory shared by every node of the job (so a
    /// build node's output is visible to a downstream test node), then removes
    /// it on completion unless `keep_workspace` is set.
    pub async fn run(&self, nodes: Vec<PipelineNode>) -> Result<(), ExecutionError> {
        // The reporter starts BEFORE any workspace I/O. The job is already
        // assigned to this agent on the control plane, and the pending-job
        // scheduler only redispatches unassigned jobs — so a failure that
        // escaped without a terminal event (e.g. an unwritable
        // --workspace-root) used to strand the job in `pending` forever. With
        // the reporter up first, even workspace allocation failures surface
        // as JobStarted → JobFailed carrying the underlying cause.
        let publisher = StatusPublisher::new(self.up_tx.clone(), self.job_id.clone());
        let mut reporter = JobReporter::start(publisher.clone()).await?;
        let cancel = CancellationToken::new();

        let (workspace, outcome) = match self.prepare_workspace().await {
            Ok(ws) => {
                let outcome = self.execute(&nodes, &publisher, &cancel, &ws).await;
                (Some(ws), outcome)
            }
            Err(e) => (None, Err(e)),
        };

        match &outcome {
            Ok(()) => reporter.commit_success(),
            Err(err) => reporter.commit_failure(format!("job failed: {err}")),
        }
        let finalize = reporter.finalize().await;

        // Always attempt cleanup, even if finalize or the run failed.
        if !self.keep_workspace
            && let Some(ws) = &workspace
            && let Err(e) = fs::remove_dir_all(ws).await
        {
            warn!(error = %e, workspace = %ws.display(), "failed to remove job workspace");
        }
        finalize?;
        outcome
    }

    /// Create and canonicalize this job's shared workspace directory. The
    /// io::Error is re-wrapped with the offending path — a bare "permission
    /// denied" in the job's failure message is not actionable.
    async fn prepare_workspace(&self) -> Result<PathBuf, ExecutionError> {
        let workspace = self.workspace_root.join(&self.job_id);
        let with_path = |e: std::io::Error| {
            ExecutionError::Workspace(std::io::Error::new(
                e.kind(),
                format!("{}: {e}", workspace.display()),
            ))
        };
        fs::create_dir_all(&workspace).await.map_err(with_path)?;
        // Canonicalize once so per-node working-dir prefix checks compare against
        // the resolved path (defeats symlink escapes).
        fs::canonicalize(&workspace).await.map_err(with_path)
    }

    async fn execute(
        &self,
        nodes: &[PipelineNode],
        publisher: &StatusPublisher,
        cancel: &CancellationToken,
        workspace: &Path,
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
                .dispatch_batch(&batch, &plan, publisher, cancel, workspace)
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
        workspace: &Path,
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
            let workspace = workspace.to_path_buf();
            let masked = self.masked_values.clone();

            running.spawn(async move {
                let result = run_node(&id, &spec, &tx, &job_id, &workspace, &masked, token).await;
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

/// Run a single node's step, streaming stdout/stderr to the agent stream.
///
/// The node runs inside the per-job `workspace` (optionally a sub-directory),
/// with a cleared+allowlisted environment and the reserved `SCYLLA_*` context
/// vars. Each node is spawned in its own process group so that on cancellation
/// the whole subtree (the shell and any children it forked) is signalled, not
/// just the immediate child.
///
/// Listens to `cancel` in parallel with `child.wait()`: if cancellation fires
/// first, the process group is killed and [`ExecutionError::Cancelled`] is returned.
async fn run_node(
    node_id: &str,
    spec: &PipelineNode,
    up_tx: &mpsc::Sender<AgentUp>,
    job_id: &str,
    workspace: &Path,
    masked: &Arc<Vec<String>>,
    cancel: CancellationToken,
) -> Result<(), ExecutionError> {
    // Resolve and create the working directory inside the workspace.
    let requested = match spec.working_dir() {
        Some(wd) => workspace.join(wd.as_str()),
        None => workspace.to_path_buf(),
    };
    if let Err(e) = fs::create_dir_all(&requested).await {
        publish_log_line(
            up_tx,
            job_id,
            node_id,
            LogStream::Stderr,
            format!("failed to create working directory: {e}"),
            masked,
        )
        .await;
        return Err(ExecutionError::Workspace(e));
    }
    // Defense in depth: canonicalize and confirm we did not escape the workspace
    // (e.g. via a symlink) before running anything.
    let cwd = fs::canonicalize(&requested)
        .await
        .map_err(ExecutionError::Workspace)?;
    if !cwd.starts_with(workspace) {
        return Err(ExecutionError::WorkspaceEscape {
            node_id: node_id.to_string(),
        });
    }

    let (mut command, program) = match build_command(node_id, spec, workspace).await {
        Ok(built) => built,
        Err(e) => {
            publish_log_line(
                up_tx,
                job_id,
                node_id,
                LogStream::Stderr,
                format!("failed to prepare step: {e}"),
                masked,
            )
            .await;
            return Err(e);
        }
    };
    configure_env(&mut command, spec, job_id, node_id, workspace);
    command
        .current_dir(&cwd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    set_process_group(&mut command);

    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            // Surface spawn failure (e.g. command-not-found, missing shell) on the
            // stderr log stream so it shows in the node logs, not just the status.
            publish_log_line(
                up_tx,
                job_id,
                node_id,
                LogStream::Stderr,
                format!("failed to spawn `{program}`: {e}"),
                masked,
            )
            .await;
            return Err(ExecutionError::Spawn(e));
        }
    };
    // Capture the group id (== child pid with process_group(0)) before wait()
    // reaps the child.
    let pgid = child.id().and_then(|id| i32::try_from(id).ok());

    // INVARIANT: stdout/stderr were configured as Stdio::piped() on the Command above.
    let stdout_handle = spawn_log_streamer(
        child.stdout.take().expect("stdout was piped"),
        LogStream::Stdout,
        node_id.to_string(),
        job_id.to_string(),
        up_tx.clone(),
        masked.clone(),
    );
    let stderr_handle = spawn_log_streamer(
        child.stderr.take().expect("stderr was piped"),
        LogStream::Stderr,
        node_id.to_string(),
        job_id.to_string(),
        up_tx.clone(),
        masked.clone(),
    );

    let wait_outcome: Result<(), ExecutionError> = tokio::select! {
        biased;
        () = cancel.cancelled() => {
            // Signal the whole group so the shell's children (cargo/make/...) die
            // too, then reap, then SIGKILL any stragglers.
            signal_group(pgid, Signal::Term);
            let _ = child.start_kill();
            let _ = child.wait().await;
            signal_group(pgid, Signal::Kill);
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

/// Build the [`Command`] for a node's step. For a script step the script is
/// materialized to `<workspace>/.scylla/<node_id>.sh` and run from a file (not
/// `-c`) so error line numbers are correct and there is no ARG_MAX limit.
/// Returns the command plus a human label of the program for error messages.
async fn build_command(
    node_id: &str,
    spec: &PipelineNode,
    workspace: &Path,
) -> Result<(Command, String), ExecutionError> {
    match spec.step() {
        Step::Exec { command, args } => {
            let mut c = Command::new(command);
            c.args(args);
            Ok((c, command.clone()))
        }
        Step::Script { script, shell } => {
            let dir = workspace.join(".scylla");
            fs::create_dir_all(&dir)
                .await
                .map_err(ExecutionError::Workspace)?;
            let path = dir.join(format!("{node_id}.sh"));
            fs::write(&path, script)
                .await
                .map_err(ExecutionError::Workspace)?;
            let c = match shell {
                Shell::Sh => {
                    let mut c = Command::new("sh");
                    c.arg("-e").arg(&path);
                    c
                }
                Shell::Bash => {
                    let mut c = Command::new("bash");
                    c.args(["--noprofile", "--norc", "-o", "pipefail", "-e"])
                        .arg(&path);
                    c
                }
            };
            let program = match shell {
                Shell::Sh => "sh",
                Shell::Bash => "bash",
            };
            Ok((c, program.to_string()))
        }
    }
}

/// Apply the node's environment to `command`: clear the inherited environment
/// (so the agent's own secrets / bearer token never leak into a job), restore a
/// minimal allowlist, overlay the node's env vars, then inject the authoritative
/// reserved `SCYLLA_*` context last.
fn configure_env(
    command: &mut Command,
    spec: &PipelineNode,
    job_id: &str,
    node_id: &str,
    workspace: &Path,
) {
    command.env_clear();
    for key in ["PATH", "HOME", "LANG", "LC_ALL"] {
        if let Ok(value) = std::env::var(key) {
            command.env(key, value);
        }
    }
    command.env("TERM", "dumb");
    // User-supplied node env overlays the allowlist. The agent only ever sees
    // resolved literals (secret refs are resolved control-plane-side).
    for ev in spec.env() {
        if let Some(value) = ev.literal_value() {
            command.env(ev.key(), value);
        }
    }
    // Reserved context, injected last so it is authoritative.
    command.env("CI", "true");
    command.env("SCYLLA_WORKSPACE", workspace);
    command.env("SCYLLA_JOB_ID", job_id);
    command.env("SCYLLA_NODE_ID", node_id);
}

/// Signal kind for [`signal_group`].
#[derive(Clone, Copy)]
enum Signal {
    Term,
    Kill,
}

/// Put the command's child into a fresh process group so its descendants can be
/// signalled as a unit. No-op on non-unix (agents run on Linux).
#[cfg(unix)]
fn set_process_group(command: &mut Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
fn set_process_group(_command: &mut Command) {}

/// Signal an entire process group by its (positive) group id. No-op on non-unix.
#[cfg(unix)]
#[allow(unsafe_code)]
fn signal_group(pgid: Option<i32>, signal: Signal) {
    if let Some(pgid) = pgid {
        let sig = match signal {
            Signal::Term => libc::SIGTERM,
            Signal::Kill => libc::SIGKILL,
        };
        // SAFETY: kill(2) with a negative pid targets the process group. Errors
        // (e.g. the group already exited) are intentionally ignored.
        unsafe {
            libc::kill(-pgid, sig);
        }
    }
}

#[cfg(not(unix))]
fn signal_group(_pgid: Option<i32>, _signal: Signal) {}

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
    masked: Arc<Vec<String>>,
) -> tokio::task::JoinHandle<()>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if !publish_log_line(&up_tx, &job_id, &node_id, stream, line, &masked).await {
                break;
            }
        }
    })
}

/// Domain log stream → the proto enum. Total: the domain has no "unspecified".
const fn log_stream_to_proto(stream: LogStream) -> common::LogStream {
    match stream {
        LogStream::Stdout => common::LogStream::Stdout,
        LogStream::Stderr => common::LogStream::Stderr,
    }
}

/// Wall-clock now as a protobuf `Timestamp`. Mirrors `scylla-api`'s
/// `convert::ts`; scylla-agent does not depend on scylla-api so it cannot
/// reuse it.
fn now_timestamp() -> Option<prost_types::Timestamp> {
    let now = Utc::now();
    Some(prost_types::Timestamp {
        seconds: now.timestamp(),
        nanos: i32::try_from(now.timestamp_subsec_nanos()).unwrap_or(0),
    })
}

/// Replace any secret-sourced value with `***` so secrets don't leak into logs.
fn redact(mut line: String, masked: &[String]) -> String {
    for secret in masked {
        if !secret.is_empty() {
            line = line.replace(secret.as_str(), "***");
        }
    }
    line
}

/// Send a single log line as a `AgentUp`. Returns `false` if the channel is
/// closed (caller should stop emitting).
async fn publish_log_line(
    up_tx: &mpsc::Sender<AgentUp>,
    job_id: &str,
    node_id: &str,
    stream: LogStream,
    line: String,
    masked: &[String],
) -> bool {
    let line = redact(line, masked);
    let log = JobLogLine {
        job_id: Some(common::JobId {
            value: job_id.to_string(),
        }),
        node_id: Some(common::NodeId {
            value: node_id.to_string(),
        }),
        stream: log_stream_to_proto(stream) as i32,
        line,
        timestamp: now_timestamp(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use scylla_core::domain::value_objects::pipeline::{EnvKey, EnvVar, NodeId, WorkingDir};
    use scylla_protocol::agent::v1::{JobStatus as ProtoJobStatus, job_status::Event};

    /// Name the oneof event variant carried by a status, for order/`contains`
    /// assertions (replaces the old integer `kind` discriminant).
    fn event_name(e: &Event) -> &'static str {
        match e {
            Event::JobStarted(_) => "job_started",
            Event::NodeStarted(_) => "node_started",
            Event::NodeCompleted(_) => "node_completed",
            Event::NodeFailed(_) => "node_failed",
            Event::NodeSkipped(_) => "node_skipped",
            Event::JobCompleted(_) => "job_completed",
            Event::JobFailed(_) => "job_failed",
        }
    }

    /// A unique temp path per test tag, keyed on pid to avoid collisions with
    /// parallel test binaries (matches the existing test's manual-temp style, so
    /// no `tempfile` dependency is pulled in).
    fn tmp_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("scylla-exec-{tag}-{}", std::process::id()))
    }

    /// Build a script (`sh`) node with optional deps, working dir and literal env.
    fn script_node(
        id: &str,
        deps: &[&str],
        script: &str,
        working_dir: Option<&str>,
        env: &[(&str, &str)],
    ) -> PipelineNode {
        let node_id = NodeId::new(id).unwrap();
        let deps = deps.iter().map(|d| NodeId::new(*d).unwrap()).collect();
        let step = Step::script(script.to_string(), Shell::Sh).unwrap();
        let working_dir = working_dir.map(|w| WorkingDir::new(w).unwrap());
        let env = env
            .iter()
            .map(|(k, v)| EnvVar::literal(EnvKey::new(*k).unwrap(), (*v).to_string()))
            .collect();
        PipelineNode::new(node_id, deps, step, working_dir, env)
    }

    /// Drain every emitted message into (status events, concatenated log lines).
    /// Safe to call right after `run()` returns: the executor awaits its log
    /// streamers before finishing, so all lines are already queued.
    fn drain(rx: &mut mpsc::Receiver<AgentUp>) -> (Vec<ProtoJobStatus>, String) {
        let mut statuses = Vec::new();
        let mut logs = String::new();
        while let Ok(msg) = rx.try_recv() {
            match msg.payload {
                Some(agent_up::Payload::Status(s)) => statuses.push(s),
                Some(agent_up::Payload::Log(l)) => {
                    logs.push_str(&l.line);
                    logs.push('\n');
                }
                None => {}
            }
        }
        (statuses, logs)
    }

    fn kinds(statuses: &[ProtoJobStatus]) -> Vec<&'static str> {
        statuses
            .iter()
            .filter_map(|s| s.event.as_ref().map(event_name))
            .collect()
    }

    #[test]
    fn redact_masks_every_occurrence_and_ignores_empty() {
        assert_eq!(
            redact("token=abc123 again abc123".into(), &["abc123".into()]),
            "token=*** again ***",
        );
        // An empty mask must be a no-op, not replace between every character.
        assert_eq!(redact("plain".into(), &[String::new()]), "plain");
        assert_eq!(
            redact("nothing here".into(), &["secret".into()]),
            "nothing here"
        );
    }

    #[tokio::test]
    async fn masked_values_are_redacted_in_job_logs() {
        let root = tmp_root("redact");
        let (tx, mut rx) = mpsc::channel(64);
        let exec = Executor::new(
            tx,
            "job-redact".into(),
            root.clone(),
            false,
            vec!["s3cr3t-value".into()],
        );
        // The secret arrives as a node env literal (as it would after control-plane
        // resolution) and is echoed; the emitted log line must be scrubbed.
        let node = script_node(
            "n1",
            &[],
            "echo \"leaking $TOKEN here\"",
            None,
            &[("TOKEN", "s3cr3t-value")],
        );
        exec.run(vec![node]).await.unwrap();
        let (_statuses, logs) = drain(&mut rx);
        let _ = std::fs::remove_dir_all(&root);

        assert!(
            logs.contains("***"),
            "masked value should be redacted; logs: {logs:?}"
        );
        assert!(
            !logs.contains("s3cr3t-value"),
            "the raw secret must never reach the log stream; logs: {logs:?}",
        );
    }

    #[tokio::test]
    async fn agent_environment_does_not_leak_into_jobs() {
        // Pick a variable in the agent's own environment that the allowlist does
        // NOT restore and the shell does not auto-set, to prove `env_clear()`
        // drops it. In a cargo test run there is always such a var (USER,
        // CARGO_*, ...); the reserved-context assertion below holds regardless.
        const KEPT: [&str; 8] = [
            "PATH", "HOME", "LANG", "LC_ALL", "TERM", "CI", "PWD", "SHLVL",
        ];
        let probe = std::env::vars()
            .map(|(k, _)| k)
            .find(|k| {
                !KEPT.contains(&k.as_str())
                    && !k.starts_with("SCYLLA_")
                    && k != "_"
                    && k != "OLDPWD"
                    && !k.is_empty()
                    && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            })
            .unwrap_or_else(|| "SCYLLA_NO_SUCH_VAR".to_string());

        let root = tmp_root("envclear");
        let (tx, mut rx) = mpsc::channel(64);
        let exec = Executor::new(tx, "job-env".into(), root.clone(), false, vec![]);
        let script = format!("echo \"LEAK=[${{{probe}}}]\"; echo \"JOB=[$SCYLLA_JOB_ID]\"");
        exec.run(vec![script_node("n1", &[], &script, None, &[])])
            .await
            .unwrap();
        let (_statuses, logs) = drain(&mut rx);
        let _ = std::fs::remove_dir_all(&root);

        assert!(
            logs.contains("LEAK=[]"),
            "the agent's own env var `{probe}` must not leak into the job; logs: {logs:?}",
        );
        assert!(
            logs.contains("JOB=[job-env]"),
            "reserved SCYLLA_JOB_ID must still be injected; logs: {logs:?}",
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn working_directory_escaping_the_workspace_is_rejected() {
        let root = tmp_root("escape");
        let outside = tmp_root("escape-outside");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&outside);
        std::fs::create_dir_all(&outside).unwrap();
        let workspace = root.join("job-escape");
        std::fs::create_dir_all(&workspace).unwrap();
        // A symlink inside the workspace that resolves outside it.
        std::os::unix::fs::symlink(&outside, workspace.join("out")).unwrap();

        let (tx, mut rx) = mpsc::channel(64);
        let exec = Executor::new(tx, "job-escape".into(), root.clone(), false, vec![]);
        let node = script_node("n1", &[], "echo hi", Some("out"), &[]);
        let result = exec.run(vec![node]).await;
        let (statuses, _logs) = drain(&mut rx);
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&outside);

        assert!(
            result.is_err(),
            "a working dir escaping the workspace must fail the job"
        );
        assert!(
            kinds(&statuses).contains(&"job_failed"),
            "the job must still report a terminal JobFailed",
        );
    }

    #[tokio::test]
    async fn a_nonzero_exit_code_maps_to_node_failed() {
        let root = tmp_root("exit");
        let (tx, mut rx) = mpsc::channel(64);
        let exec = Executor::new(tx, "job-exit".into(), root.clone(), false, vec![]);
        let err = exec
            .run(vec![script_node("n1", &[], "exit 3", None, &[])])
            .await
            .unwrap_err();
        let (statuses, _logs) = drain(&mut rx);
        let _ = std::fs::remove_dir_all(&root);

        assert!(
            matches!(err, ExecutionError::NodeFailed { exit_code: 3, .. }),
            "exit code must be preserved, got {err:?}",
        );
        assert!(kinds(&statuses).contains(&"job_failed"));
    }

    #[tokio::test]
    async fn a_dependent_node_is_skipped_when_its_dependency_fails() {
        let root = tmp_root("skip");
        let (tx, mut rx) = mpsc::channel(64);
        let exec = Executor::new(tx, "job-skip".into(), root.clone(), false, vec![]);
        let failing = script_node("build", &[], "exit 1", None, &[]);
        let dependent = script_node("test", &["build"], "echo should-not-run", None, &[]);
        let _ = exec.run(vec![failing, dependent]).await;
        let (statuses, logs) = drain(&mut rx);
        let _ = std::fs::remove_dir_all(&root);

        assert!(
            kinds(&statuses).contains(&"node_skipped"),
            "the dependent node must be skipped once its dependency fails",
        );
        assert!(
            !logs.contains("should-not-run"),
            "a skipped node must never execute; logs: {logs:?}",
        );
        assert!(kinds(&statuses).contains(&"job_failed"));
    }

    /// Regression: a workspace root that cannot be created (the bug was a
    /// missing/unwritable --workspace-root) must still bracket the job with
    /// JobStarted → JobFailed on the up-stream, carrying the offending path.
    /// Before the fix the executor returned before the reporter existed and
    /// the control plane never heard about the job again.
    #[tokio::test]
    async fn workspace_failure_still_reports_started_and_failed() {
        // A root nested under a regular FILE can never be created.
        let blocker = std::env::temp_dir().join(format!("scylla-exec-test-{}", std::process::id()));
        tokio::fs::write(&blocker, b"x").await.unwrap();
        let root = blocker.join("ws");

        let (tx, mut rx) = mpsc::channel(16);
        let exec = Executor::new(tx, "job-1".into(), root, false, vec![]);
        let result = exec.run(vec![]).await;
        let _ = tokio::fs::remove_file(&blocker).await;
        assert!(result.is_err(), "workspace creation must fail");

        let mut statuses = vec![];
        while let Ok(msg) = rx.try_recv() {
            if let Some(agent_up::Payload::Status(s)) = msg.payload {
                statuses.push(s);
            }
        }
        assert_eq!(
            kinds(&statuses),
            vec!["job_started", "job_failed"],
            "a workspace failure must still produce exactly one terminal event"
        );
        let last_error = match statuses[1].event.as_ref() {
            Some(Event::JobFailed(f)) => f.error.as_str(),
            other => panic!("expected a terminal JobFailed, got {other:?}"),
        };
        assert!(
            last_error.contains("scylla-exec-test"),
            "the failure message must carry the offending path, got: {last_error}",
        );
    }
}
