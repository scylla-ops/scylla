use serde::{Deserialize, Serialize};

use super::LogStream;

/// Log line published by the agent on `scylla.jobs.logs.{job_id}.{node_id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLogLine {
    pub node_id: String,
    pub stream: LogStream,
    pub line: String,
    pub timestamp: String,
}
