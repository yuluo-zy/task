use diesel_derive_enum::DbEnum;

#[derive(DbEnum, Debug, Eq, PartialEq, Clone)]
pub enum FangTaskState {
    New,
    InProgress,
    Failed,
    Finished,
}

table! {
    use super::FangTaskStateMapping;
    use diesel::sql_types::Jsonb;
    use diesel::sql_types::Nullable;
    use diesel::sql_types::Text;
    use diesel::sql_types::Timestamptz;
    use diesel::sql_types::Uuid;


    fang_tasks (id) {
        id -> Uuid,
        metadata -> Jsonb,
        error_message -> Nullable<Text>,
        state -> FangTaskStateMapping,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}