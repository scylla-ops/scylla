//! centralized declaration of SurrealDB tables
//!
//! all tables are defined here, each with utility functions

use surrealdb::{RecordId, RecordIdKey};

macro_rules! declare_table {
    (
        $(#[$meta:meta])*
        $mod_name:ident => $table_name:literal
    ) => {
        $(#[$meta])*
        #[allow(dead_code)]
        pub mod $mod_name {
            use super::*;

            pub const TABLE: &str = $table_name;

            #[inline]
            pub fn to_record_id(key: RecordIdKey) -> RecordId {
                RecordId::from((TABLE, key.to_string()))
            }

            #[inline]
            pub fn to_record_id_ref(key: &RecordIdKey) -> RecordId {
                RecordId::from((TABLE, key.to_string()))
            }
        }
    };
}

declare_table!(
    users => "users"
);

declare_table!(
    organizations => "organizations"
);

declare_table!(
    user_organization => "user_organization"
);

declare_table!(
    jobs => "jobs"
);

declare_table!(
    pipelines => "pipelines"
);

declare_table!(
    projects => "projects"
);

declare_table!(
    user_project => "user_project"
);

declare_table!(
    casbin_rules => "casbin_rules"
);
