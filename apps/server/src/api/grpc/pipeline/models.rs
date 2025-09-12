use chrono::{DateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};
use uuid::Uuid;

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::pipelines)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPipeline<'a> {
    pub content: &'a str,
}

#[derive(Identifiable, Queryable, Debug)]
#[diesel(table_name = crate::database::schema::pipelines)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PipelineRecord {
    pub id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
