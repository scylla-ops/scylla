use chrono::{DateTime, Utc};
use protocol::pipeline::Pipeline;
use protocol::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct NewPipeline {
    pub content: Pipeline,
}

#[derive(Deserialize)]
pub struct PipelineRecord {
    #[cfg(feature = "surreal")]
    pub id: surrealdb::RecordId,
    pub content: Pipeline,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub struct PipelinePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Pipeline>,
}
