use crate::api::grpc::pipeline::PipelineRecord;
use chrono::{DateTime, Utc};
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::pipeline_snapshots)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPipelineSnapshot<'a> {
    pub pipeline_id: Uuid,
    pub content: &'a str,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug)]
#[diesel(table_name = crate::database::schema::pipeline_snapshots)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(PipelineRecord, foreign_key = pipeline_id))]
pub struct PipelineSnapshotRecord {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}
