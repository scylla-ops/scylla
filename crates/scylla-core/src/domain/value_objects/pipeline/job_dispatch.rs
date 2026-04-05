use serde::{Deserialize, Serialize};

use crate::domain::entities::PipelineNode;

/// Payload published by scylla-api on `scylla.jobs.dispatch`.
/// Contains everything the agent needs to execute a pipeline job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDispatch {
    pub job_id: String,
    pub pipeline_id: String,
    pub nodes: Vec<PipelineNode>,
}
