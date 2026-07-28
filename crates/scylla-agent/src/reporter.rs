//! Status reporting + terminal event guard.
//!
//! [`StatusPublisher`] wraps the agent up-stream channel and emits
//! [`JobEvent`]s as `AgentUp` status messages. [`JobReporter`] wraps it with
//! scope-exit semantics: `JobStarted` fires on construction, and exactly one of
//! `JobCompleted`/`JobFailed` fires via [`JobReporter::finalize`] at the end of
//! the run — regardless of which path the executor took.

use scylla_core::JobEvent;
use scylla_protocol::agent::v1::job_status::{
    JobCompleted, JobFailed, JobStarted, NodeCompleted, NodeFailed, NodeSkipped, NodeStarted,
};
use scylla_protocol::agent::v1::{AgentUp, JobStatus, agent_up, job_status};
use scylla_protocol::common::v1 as common;
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
/// Each arm builds exactly the oneof variant that carries this event's fields —
/// a job-level event has no node, a node event always names one.
fn job_event_to_status(job_id: &str, event: JobEvent) -> JobStatus {
    use job_status::Event;
    let node = |value: String| Some(common::NodeId { value });
    let event = match event {
        JobEvent::JobStarted => Event::JobStarted(JobStarted {}),
        JobEvent::NodeStarted { node_id } => Event::NodeStarted(NodeStarted {
            node_id: node(node_id),
        }),
        JobEvent::NodeCompleted { node_id } => Event::NodeCompleted(NodeCompleted {
            node_id: node(node_id),
        }),
        JobEvent::NodeFailed { node_id, error } => Event::NodeFailed(NodeFailed {
            node_id: node(node_id),
            error,
        }),
        JobEvent::NodeSkipped { node_id } => Event::NodeSkipped(NodeSkipped {
            node_id: node(node_id),
        }),
        JobEvent::JobCompleted => Event::JobCompleted(JobCompleted {}),
        JobEvent::JobFailed { error } => Event::JobFailed(JobFailed { error }),
    };
    JobStatus {
        job_id: Some(common::JobId {
            value: job_id.to_string(),
        }),
        event: Some(event),
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
