// @generated automatically by Diesel CLI.

diesel::table! {
    fang_tasks (id) {
        id -> Varchar,
        metadata -> Json,
        error_message -> Nullable<Text>,
        state -> Char,
        task_type -> Varchar,
        uniq_hash -> Nullable<Char>,
        retries -> Integer,
        scheduled_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
