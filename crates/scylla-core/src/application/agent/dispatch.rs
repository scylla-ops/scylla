use crate::domain::entities::PipelineNode;
use serde::{Deserialize, Serialize};

/// Everything an agent needs to execute a pipeline job, handed to a connected
/// agent through the [`AgentDispatch`](crate::application::AgentDispatch) port.
/// This is an application/transport payload (the port's data contract), not a
/// domain value object — it carries the resolved [`PipelineNode`]s plus the
/// job/pipeline identifiers the wire form needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDispatch {
    pub job_id: String,
    pub pipeline_id: String,
    pub nodes: Vec<PipelineNode>,
}
