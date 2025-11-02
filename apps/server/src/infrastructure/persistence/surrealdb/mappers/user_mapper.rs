use crate::domain::entities::User;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{UserId, Username};
use crate::infrastructure::persistence::surrealdb::models::{UserInsert, UserRecord, UserUpdate};
use chrono::DateTime;

/// Mapper between User domain entity and database records
pub struct UserMapper;

impl UserMapper {
    /// Convert database record to domain entity
    pub fn to_domain(record: UserRecord) -> DomainResult<User> {
        let id = UserId::new(record.id.key().to_string());
        let username = Username::new(record.username)?;

        User::new(
            id,
            username,
            record.password_hash,
            record.is_active,
            DateTime::from(record.created_at),
            DateTime::from(record.updated_at),
        )
    }

    /// Convert domain entity to insert record
    pub fn to_insert(user: &User) -> UserInsert {
        UserInsert {
            username: user.username().to_string(),
            password_hash: user.password_hash().to_string(),
            is_active: user.is_active(),
        }
    }

    /// Convert domain entity to update record
    pub fn to_update(user: &User) -> UserUpdate {
        UserUpdate {
            username: user.username().to_string(),
            is_active: user.is_active(),
        }
    }
}
