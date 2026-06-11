use serde::{Deserialize, Serialize};

/// A job lifecycle event reported by a running agent, consumed by
/// [`JobUseCases::record_status`](crate::application::JobUseCases::record_status)
/// to drive the job's state machine. This is an **application command**, not a
/// domain value object — the transport adapter (gRPC) maps its wire form to this
/// vocabulary; the use case then applies it to the `Job` aggregate.
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
