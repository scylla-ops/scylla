use serde::{Deserialize, Serialize};

/// Status event published by the agent on `scylla.jobs.status.{job_id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusUpdate {
    pub job_id: String,
    pub event: JobEvent,
}

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
