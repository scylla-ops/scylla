// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "execution_status"))]
    pub struct ExecutionStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ExecutionStatus;

    jobs (id) {
        id -> Uuid,
        pipeline_snapshot_id -> Uuid,
        status -> ExecutionStatus,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    pipeline_snapshots (id) {
        id -> Uuid,
        pipeline_id -> Uuid,
        content -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    pipelines (id) {
        id -> Uuid,
        content -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ExecutionStatus;

    stages (id) {
        id -> Uuid,
        job_id -> Uuid,
        status -> ExecutionStatus,
        position -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ExecutionStatus;

    steps (id) {
        id -> Uuid,
        stage_id -> Uuid,
        status -> ExecutionStatus,
        position -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    teams (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        username -> Varchar,
        password_hash -> Text,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(jobs -> pipeline_snapshots (pipeline_snapshot_id));
diesel::joinable!(pipeline_snapshots -> pipelines (pipeline_id));
diesel::joinable!(stages -> jobs (job_id));
diesel::joinable!(steps -> stages (stage_id));

diesel::allow_tables_to_appear_in_same_query!(
    jobs,
    pipeline_snapshots,
    pipelines,
    stages,
    steps,
    teams,
    users,
);
