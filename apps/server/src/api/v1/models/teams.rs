use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::database::schema::teams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Team {
    pub id: uuid::Uuid,
    #[diesel(column_name = "name")]
    pub name: String,
    #[diesel(column_name = "created_at")]
    pub created_at: DateTime<Utc>,
    #[diesel(column_name = "updated_at")]
    pub updated_at: DateTime<Utc>,
}
