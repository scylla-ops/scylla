use diesel::{AsChangeset, Insertable};
use diesel_derive_enum::DbEnum;
use protocol::services::orchestrator::EventKind;
use uuid::Uuid;

#[derive(DbEnum, Debug)]
#[ExistingTypePath = "crate::database::schema::sql_types::ExecutionStatus"]
#[DbValueStyle = "snake_case"]
pub enum ExecutionStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

impl From<EventKind> for ExecutionStatus {
    fn from(value: EventKind) -> Self {
        match value {
            EventKind::Queued => ExecutionStatus::Queued,
            EventKind::Running => ExecutionStatus::Running,
            EventKind::Succeeded => ExecutionStatus::Succeeded,
            EventKind::Failed => ExecutionStatus::Failed,
            EventKind::Canceled => ExecutionStatus::Canceled,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::jobs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewJob {
    pub id: Uuid,
    pub pipeline_snapshot_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::stages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewStage {
    pub id: Uuid,
    pub job_id: Uuid,
    pub position: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::steps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewStep {
    pub id: Uuid,
    pub stage_id: Uuid,
    pub position: i32,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::database::schema::jobs)]
pub struct JobStatusUpdate {
    pub status: ExecutionStatus,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::database::schema::stages)]
pub struct StageStatusUpdate {
    pub status: ExecutionStatus,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::database::schema::steps)]
pub struct StepStatusUpdate {
    pub status: ExecutionStatus,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
