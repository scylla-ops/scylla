use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::database::schema::commands)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Command {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Command {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            command,
            args,
            status: "submitted".to_string(),
            created_at: Utc::now(),
            updated_at: None,
        }
    }
}
