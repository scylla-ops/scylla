// @generated automatically by Diesel CLI.

diesel::table! {
    commands (id) {
        id -> Varchar,
        command -> Text,
        args -> Array<Text>,
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}
