// @generated automatically by Diesel CLI.

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

diesel::joinable!(pipeline_snapshots -> pipelines (pipeline_id));

diesel::allow_tables_to_appear_in_same_query!(pipeline_snapshots, pipelines, teams, users,);
