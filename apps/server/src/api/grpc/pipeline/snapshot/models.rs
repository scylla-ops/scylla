use chrono::{DateTime, Utc};
use protocol::pipeline::Pipeline;
use protocol::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct NewPipelineSnapshot {
    #[cfg(feature = "surreal")]
    pub pipeline: surrealdb::RecordId,
    pub content: Pipeline,
}

#[derive(Deserialize, Debug)]
pub struct PipelineSnapshotRecord {
    #[cfg(feature = "surreal")]
    pub id: surrealdb::RecordId,
    #[cfg(feature = "surreal")]
    pub pipeline: surrealdb::RecordId,
    pub content: Pipeline,
    pub created_at: DateTime<Utc>,
}
