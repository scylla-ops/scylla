use serde::{Deserialize, Serialize};

/// A job lifecycle event reported by a running agent, consumed by the control
/// plane's `JobUseCases::record_status` to drive the job's state machine.
///
/// Not a domain value object: it is the command vocabulary shared by the two
/// binaries. The agent emits it, the gRPC adapter maps it to and from its wire
/// form, and the use case applies it to the `Job` aggregate. It lives in the
/// kernel precisely because both sides need to agree on it; duplicating it would
/// let them drift apart silently.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JobEvent {
    JobStarted,
    NodeStarted { node_id: String },
    NodeCompleted { node_id: String },
    NodeFailed { node_id: String, error: String },
    NodeSkipped { node_id: String },
    JobCompleted,
    JobFailed { error: String },
}
