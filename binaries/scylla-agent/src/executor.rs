use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use scylla_domain::JobEvent;
use scylla_domain::domain::job::LogStream;
use scylla_domain::domain::pipeline::{DagPlan, PipelineNode, Shell, Step};
use scylla_proto::agent::v1::{AgentUp, JobLogLine, agent_up};
use scylla_proto::common::v1 as common;

use crate::error::ExecutionError;

use crate::reporter::{JobReporter, StatusPublisher};

pub struct Executor {
    up_tx: mpsc::Sender<AgentUp>,
    job_id: String,
    workspace_root: PathBuf,
    keep_workspace: bool,
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

    pub async fn run(&self, nodes: Vec<PipelineNode>) -> Result<(), ExecutionError> {
        // The reporter starts before any workspace I/O: a failure with no terminal event strands the job in `pending`.
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

        if !self.keep_workspace
            && let Some(ws) = &workspace
            && let Err(e) = fs::remove_dir_all(ws).await
        {
            warn!(error = %e, workspace = %ws.display(), "failed to remove job workspace");
        }
        finalize?;
        outcome
    }

    async fn prepare_workspace(&self) -> Result<PathBuf, ExecutionError> {
        let workspace = self.workspace_root.join(&self.job_id);
        let with_path = |e: std::io::Error| {
            ExecutionError::Workspace(std::io::Error::new(
                e.kind(),
                format!("{}: {e}", workspace.display()),
            ))
        };
        fs::create_dir_all(&workspace).await.map_err(with_path)?;
        // Canonicalize once so the per-node prefix checks defeat symlink escapes.
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

/// Own process group per node so cancellation signals the whole subtree, not just the shell.
async fn run_node(
    node_id: &str,
    spec: &PipelineNode,
    up_tx: &mpsc::Sender<AgentUp>,
    job_id: &str,
    workspace: &Path,
    masked: &Arc<Vec<String>>,
    cancel: CancellationToken,
) -> Result<(), ExecutionError> {
    let requested = match spec.working_dir() {
        Some(wd) => {
            // Before anything is created: `create_dir_all` would materialize an escaping path before the canonical check.
            if Path::new(wd.as_str())
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::RootDir))
            {
                return Err(ExecutionError::WorkspaceEscape {
                    node_id: node_id.to_string(),
                });
            }
            workspace.join(wd.as_str())
        }
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
    // Canonical check: a symlink inside the workspace may resolve outside it.
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
    // Capture before wait() reaps the child.
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

    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

    wait_outcome
}

/// Scripts run from a file, not `-c`: correct line numbers and no ARG_MAX limit.
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

/// The inherited environment is cleared so the agent's own token and secrets never reach a job.
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
    for ev in spec.env() {
        if let Some(value) = ev.literal_value() {
            command.env(ev.key(), value);
        }
    }
    // Injected last so it is authoritative.
    command.env("CI", "true");
    command.env("SCYLLA_WORKSPACE", workspace);
    command.env("SCYLLA_JOB_ID", job_id);
    command.env("SCYLLA_NODE_ID", node_id);
}

#[derive(Clone, Copy)]
enum Signal {
    Term,
    Kill,
}

#[cfg(unix)]
fn set_process_group(command: &mut Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
fn set_process_group(_command: &mut Command) {}

#[cfg(unix)]
#[allow(unsafe_code)]
fn signal_group(pgid: Option<i32>, signal: Signal) {
    if let Some(pgid) = pgid {
        let sig = match signal {
            Signal::Term => libc::SIGTERM,
            Signal::Kill => libc::SIGKILL,
        };
        // SAFETY: kill(2) with a negative pid targets the process group. Errors
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

fn redact(mut line: String, masked: &[String]) -> String {
    for secret in masked {
        if !secret.is_empty() {
            line = line.replace(secret.as_str(), "***");
        }
    }
    line
}

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
        stream: scylla_proto::convert::log_stream_to_proto(stream) as i32,
        line,
        timestamp: scylla_proto::convert::timestamp(Utc::now()),
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
    use scylla_domain::domain::pipeline::{EnvKey, EnvVar, NodeId, WorkingDir};
    use scylla_proto::agent::v1::{JobStatus as ProtoJobStatus, job_status::Event};

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

    fn tmp_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("scylla-exec-{tag}-{}", std::process::id()))
    }

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
                Some(agent_up::Payload::Hello(_)) | None => {}
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

    #[tokio::test]
    async fn workspace_failure_still_reports_started_and_failed() {
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
