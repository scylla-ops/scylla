use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::database::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: uuid::Uuid,
    #[diesel(column_name = "username")]
    pub username: String,
    #[diesel(column_name = "password_hash")]
    pub password_hash: String,
    #[diesel(column_name = "is_active")]
    pub is_active: bool,
    #[diesel(column_name = "created_at")]
    pub created_at: DateTime<Utc>,
    #[diesel(column_name = "updated_at")]
    pub updated_at: DateTime<Utc>,
}
