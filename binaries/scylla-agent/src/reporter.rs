use scylla_domain::JobEvent;
use scylla_proto::agent::v1::{AgentUp, agent_up};
use tokio::sync::mpsc;

use crate::error::ExecutionError;

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
        let status = scylla_proto::convert::job_event_to_status(&self.job_id, event);
        self.up_tx
            .send(AgentUp {
                payload: Some(agent_up::Payload::Status(status)),
            })
            .await
            .map_err(|e| ExecutionError::Publish(e.to_string()))
    }
}

enum JobOutcome {
    Pending,
    Success,
    Failure(String),
}

pub struct JobReporter {
    publisher: StatusPublisher,
    outcome: JobOutcome,
}

impl JobReporter {
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
