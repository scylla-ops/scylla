use protocol::job::JobData;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

#[derive(Serialize, Deserialize)]
pub struct NewJob {
    #[cfg(feature = "surreal")]
    pub snapshot: RecordId,
    pub content: JobData,
}

#[derive(Serialize, Deserialize)]
pub struct JobRecord {
    #[cfg(feature = "surreal")]
    pub id: RecordId,
    #[cfg(feature = "surreal")]
    pub snapshot: RecordId,
    pub content: JobData,
}

#[derive(Serialize)]
pub struct JobUpdate {
    pub content: JobData,
}
