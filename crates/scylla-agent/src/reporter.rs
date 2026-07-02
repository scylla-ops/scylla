//! Status reporting + terminal event guard.
//!
//! [`StatusPublisher`] wraps the agent up-stream channel and emits
//! [`JobEvent`]s as `AgentUp` status messages. [`JobReporter`] wraps it with
//! scope-exit semantics: `JobStarted` fires on construction, and exactly one of
//! `JobCompleted`/`JobFailed` fires via [`JobReporter::finalize`] at the end of
//! the run — regardless of which path the executor took.

use scylla_core::application::JobEvent;
use scylla_protocol::services::agent::{AgentUp, JobEventKind, JobStatus, agent_up};
use scylla_protocol::services::common;
use tokio::sync::mpsc;

use crate::error::ExecutionError;

/// Emits [`JobEvent`]s as `AgentUp` status messages on the agent stream.
/// Cheaply cloneable.
#[derive(Clone)]
pub struct StatusPublisher {
    up_tx: mpsc::Sender<AgentUp>,
    job_id: String,
}

impl StatusPublisher {
    #[must_use]
    pub fn new(up_tx: mpsc::Sender<AgentUp>, job_id: String) -> Self {
        Self { up_tx, job_id }
    }

    pub async fn emit(&self, event: JobEvent) -> Result<(), ExecutionError> {
        let status = job_event_to_status(&self.job_id, event);
        self.up_tx
            .send(AgentUp {
                payload: Some(agent_up::Payload::Status(status)),
            })
            .await
            .map_err(|e| ExecutionError::Publish(e.to_string()))
    }
}

/// Map a domain [`JobEvent`] to the proto [`JobStatus`] sent over the stream.
fn job_event_to_status(job_id: &str, event: JobEvent) -> JobStatus {
    let (kind, node_id, error) = match event {
        JobEvent::JobStarted => (JobEventKind::JobStarted, String::new(), String::new()),
        JobEvent::NodeStarted { node_id } => (JobEventKind::NodeStarted, node_id, String::new()),
        JobEvent::NodeCompleted { node_id } => {
            (JobEventKind::NodeCompleted, node_id, String::new())
        }
        JobEvent::NodeFailed { node_id, error } => (JobEventKind::NodeFailed, node_id, error),
        JobEvent::NodeSkipped { node_id } => (JobEventKind::NodeSkipped, node_id, String::new()),
        JobEvent::JobCompleted => (JobEventKind::JobCompleted, String::new(), String::new()),
        JobEvent::JobFailed { error } => (JobEventKind::JobFailed, String::new(), error),
    };
    JobStatus {
        job_id: Some(common::JobId {
            value: job_id.to_string(),
        }),
        kind: kind as i32,
        // Empty node_id means a job-level event (no node) — send it unset.
        node_id: (!node_id.is_empty()).then_some(common::NodeId { value: node_id }),
        error,
    }
}

enum JobOutcome {
    Pending,
    Success,
    Failure(String),
}

/// Guards the terminal lifecycle of a job: emits `JobStarted` on creation and a
/// single `JobCompleted`/`JobFailed` on [`JobReporter::finalize`].
pub struct JobReporter {
    publisher: StatusPublisher,
    outcome: JobOutcome,
}

impl JobReporter {
    /// Start a new job: emits `JobStarted`.
    pub async fn start(publisher: StatusPublisher) -> Result<Self, ExecutionError> {
        publisher.emit(JobEvent::JobStarted).await?;
        Ok(Self {
            publisher,
            outcome: JobOutcome::Pending,
        })
    }

    pub fn commit_success(&mut self) {
        self.outcome = JobOutcome::Success;
    }

    pub fn commit_failure(&mut self, error: String) {
        self.outcome = JobOutcome::Failure(error);
    }

    /// Emit the terminal event (`JobCompleted` / `JobFailed`) and consume the guard.
    pub async fn finalize(self) -> Result<(), ExecutionError> {
        let event = match self.outcome {
            JobOutcome::Success => JobEvent::JobCompleted,
            JobOutcome::Failure(error) => JobEvent::JobFailed { error },
            JobOutcome::Pending => JobEvent::JobFailed {
                error: "executor exited without committing outcome".into(),
            },
        };
        self.publisher.emit(event).await
    }
}
