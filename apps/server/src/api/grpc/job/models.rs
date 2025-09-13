use diesel::Insertable;
use diesel_derive_enum::DbEnum;
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

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::jobs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewJob {
    pub pipeline_snapshot_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::stages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewStage {
    pub job_id: Uuid,
    pub position: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::steps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewStep {
    pub stage_id: Uuid,
    pub position: i32,
}
