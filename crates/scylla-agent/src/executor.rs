use std::collections::{BTreeSet, HashMap};

use chrono::Utc;
use hermes_broker_proto::PublishRequest;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

use crate::error::ExecutionError;
use scylla_core::domain::entities::PipelineNode;
use scylla_core::domain::value_objects::job::{JobEvent, JobLogLine, JobStatusUpdate, LogStream};

/// Executes a pipeline DAG by walking nodes in topological order,
/// spawning parallel tasks for independent nodes.
pub struct Executor {
    /// Channel to send publish requests to the broker.
    publish_tx: mpsc::Sender<PublishRequest>,
    /// Subject for status updates (from reply_to).
    status_subject: String,
    /// Base subject for logs: `scylla.jobs.logs.{job_id}`.
    logs_subject: String,
    job_id: String,
}

impl Executor {
    pub fn new(
        publish_tx: mpsc::Sender<PublishRequest>,
        status_subject: String,
        job_id: String,
    ) -> Self {
        let logs_subject = format!("scylla.jobs.logs.{}", &job_id);
        Self {
            publish_tx,
            status_subject,
            logs_subject,
            job_id,
        }
    }

    /// Execute all nodes in the DAG, respecting dependencies.
    pub async fn run(&self, nodes: Vec<PipelineNode>) -> Result<(), ExecutionError> {
        self.publish_status(JobEvent::JobStarted).await?;

        let node_map: HashMap<&str, &PipelineNode> =
            nodes.iter().map(|n| (n.id().as_str(), n)).collect();

        // Build in-degree map
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

        for node in &nodes {
            in_degree.entry(node.id().as_str()).or_insert(0);
            for dep in node.deps() {
                *in_degree.entry(node.id().as_str()).or_insert(0) += 1;
                dependents
                    .entry(dep.as_str())
                    .or_default()
                    .push(node.id().as_str());
            }
        }

        // Collect initially ready nodes (in_degree == 0), sorted for determinism
        let mut ready: BTreeSet<&str> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut completed_count = 0;
        let total = nodes.len();

        while !ready.is_empty() {
            let batch: Vec<&str> = ready.iter().copied().collect();
            ready.clear();

            // Report all nodes in this batch as started
            for node_id in &batch {
                self.publish_status(JobEvent::NodeStarted {
                    node_id: node_id.to_string(),
                })
                .await?;
            }

            let mut join_set = JoinSet::new();

            for node_id in &batch {
                let spec = node_map[node_id];
                let command = spec.command().to_string();
                let args = spec.args().to_vec();
                let node_id_owned = node_id.to_string();
                let publish_tx = self.publish_tx.clone();
                let logs_subject = self.logs_subject.clone();

                join_set.spawn(async move {
                    let result =
                        run_node(&node_id_owned, &command, &args, &publish_tx, &logs_subject).await;
                    (node_id_owned, result)
                });
            }

            // Wait for all nodes in this batch to complete
            while let Some(result) = join_set.join_next().await {
                let (node_id, outcome) = result.expect("task panicked");

                match outcome {
                    Ok(()) => {
                        info!(node_id = %node_id, "node completed");
                        self.publish_status(JobEvent::NodeCompleted {
                            node_id: node_id.clone(),
                        })
                        .await?;

                        completed_count += 1;

                        // Decrement in-degree of dependents
                        if let Some(deps) = dependents.get(node_id.as_str()) {
                            for &dep_id in deps {
                                if let Some(deg) = in_degree.get_mut(dep_id) {
                                    *deg -= 1;
                                    if *deg == 0 {
                                        ready.insert(dep_id);
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        error!(node_id = %node_id, error = %err, "node failed");
                        self.publish_status(JobEvent::NodeFailed {
                            node_id: node_id.clone(),
                            error: err.to_string(),
                        })
                        .await?;

                        self.publish_status(JobEvent::JobFailed {
                            error: format!("node {node_id} failed: {err}"),
                        })
                        .await?;

                        return Err(err);
                    }
                }
            }
        }

        if completed_count == total {
            self.publish_status(JobEvent::JobCompleted).await?;
        } else {
            self.publish_status(JobEvent::JobFailed {
                error: "not all nodes completed (possible cycle or missing dependency)".into(),
            })
            .await?;
        }

        Ok(())
    }

    async fn publish_status(&self, event: JobEvent) -> Result<(), ExecutionError> {
        let update = JobStatusUpdate {
            job_id: self.job_id.clone(),
            event,
        };
        let payload = serde_json::to_vec(&update).expect("serialization cannot fail");

        self.publish_tx
            .send(PublishRequest {
                subject: self.status_subject.clone(),
                payload,
                reply_to: String::new(),
            })
            .await
            .map_err(|e| ExecutionError::Publish(e.to_string()))
    }
}

/// Run a single node's command, streaming stdout/stderr to the broker.
async fn run_node(
    node_id: &str,
    command: &str,
    args: &[String],
    publish_tx: &mpsc::Sender<PublishRequest>,
    logs_subject: &str,
) -> Result<(), ExecutionError> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(ExecutionError::Spawn)?;

    let log_subject = format!("{logs_subject}.{node_id}");

    // Stream stdout
    let stdout = child.stdout.take().expect("stdout was piped");
    let stdout_tx = publish_tx.clone();
    let stdout_subject = log_subject.clone();
    let stdout_node_id = node_id.to_string();
    let stdout_handle = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let log_line = JobLogLine {
                node_id: stdout_node_id.clone(),
                stream: LogStream::Stdout,
                line,
                timestamp: Utc::now().to_rfc3339(),
            };
            let payload = serde_json::to_vec(&log_line).expect("serialization cannot fail");
            if stdout_tx
                .send(PublishRequest {
                    subject: stdout_subject.clone(),
                    payload,
                    reply_to: String::new(),
                })
                .await
                .is_err()
            {
                warn!("log publish channel closed");
                break;
            }
        }
    });

    // Stream stderr
    let stderr = child.stderr.take().expect("stderr was piped");
    let stderr_tx = publish_tx.clone();
    let stderr_subject = log_subject;
    let stderr_node_id = node_id.to_string();
    let stderr_handle = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let log_line = JobLogLine {
                node_id: stderr_node_id.clone(),
                stream: LogStream::Stderr,
                line,
                timestamp: Utc::now().to_rfc3339(),
            };
            let payload = serde_json::to_vec(&log_line).expect("serialization cannot fail");
            if stderr_tx
                .send(PublishRequest {
                    subject: stderr_subject.clone(),
                    payload,
                    reply_to: String::new(),
                })
                .await
                .is_err()
            {
                warn!("log publish channel closed");
                break;
            }
        }
    });

    let status = child.wait().await.map_err(ExecutionError::Spawn)?;

    // Wait for log streams to flush
    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

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
