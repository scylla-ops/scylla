//! Status publishing + terminal event guard.
//!
//! [`StatusPublisher`] is a thin wrapper around the broker channel that emits
//! [`JobEvent`]s on the correct subject. [`JobReporter`] wraps it with
//! scope-exit semantics: `JobStarted` fires on construction, and exactly one
//! of `JobCompleted`/`JobFailed` fires via [`JobReporter::finalize`] at the
//! end of the run — regardless of which path the executor took.

use hermes_broker_proto::PublishRequest;
use scylla_core::domain::value_objects::job::{JobEvent, JobStatusUpdate};
use tokio::sync::mpsc;

use crate::error::ExecutionError;

/// Publishes [`JobEvent`]s to the broker via the status subject.
///
/// Cheaply cloneable — all fields are `Arc`/`String` clones.
#[derive(Clone)]
pub struct StatusPublisher {
    publish_tx: mpsc::Sender<PublishRequest>,
    status_subject: String,
    job_id: String,
}

impl StatusPublisher {
    #[must_use]
    pub fn new(
        publish_tx: mpsc::Sender<PublishRequest>,
        status_subject: String,
        job_id: String,
    ) -> Self {
        Self {
            publish_tx,
            status_subject,
            job_id,
        }
    }

    pub async fn emit(&self, event: JobEvent) -> Result<(), ExecutionError> {
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

enum JobOutcome {
    Pending,
    Success,
    Failure(String),
}

/// Guards the terminal lifecycle of a job: emits `JobStarted` on creation and
/// a single `JobCompleted`/`JobFailed` on [`JobReporter::finalize`].
///
/// The executor consumes the reporter at scope exit:
/// ```ignore
/// let reporter = JobReporter::start(publisher).await?;
/// let outcome = async { /* run body */ }.await;
/// match &outcome {
///     Ok(())  => reporter.commit_success(),
///     Err(e)  => reporter.commit_failure(e.to_string()),
/// }
/// reporter.finalize().await?;
/// outcome
/// ```
pub struct JobReporter {
    publisher: StatusPublisher,
    outcome: JobOutcome,
}

impl JobReporter {
    /// Start a new job: emits `JobStarted` on the status subject.
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

    #[must_use]
    pub fn publisher(&self) -> StatusPublisher {
        self.publisher.clone()
    }

    /// Emit the terminal event (`JobCompleted` or `JobFailed`) and consume the guard.
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
