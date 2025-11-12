use crate::domain::entities::User;
use crate::domain::value_objects::{UserId, Username};
use crate::domain::{DomainError, DomainResult};
use crate::infrastructure::persistence::surrealdb::models::{UserInsert, UserRecord, UserUpdate};
use chrono::DateTime;
use std::convert::{From, TryFrom};

impl TryFrom<UserRecord> for User {
    type Error = DomainError;

    /// Convert database record to domain entity
    fn try_from(record: UserRecord) -> DomainResult<Self> {
        let id = UserId::new(record.id.key().to_string());
        let username = Username::try_from(record.username)?;

        Ok(User::new(
            id,
            username,
            record.password_hash,
            record.is_active,
            DateTime::from(record.created_at),
            DateTime::from(record.updated_at),
        ))
    }
}

impl From<&User> for UserInsert {
    /// Convert domain entity to insert record
    fn from(user: &User) -> Self {
        UserInsert {
            username: user.username().to_string(),
            password_hash: user.password_hash().to_string(),
            is_active: user.is_active(),
        }
    }
}

impl From<&User> for UserUpdate {
    /// Convert domain entity to update record
    fn from(user: &User) -> Self {
        UserUpdate {
            username: user.username().to_string(),
            is_active: user.is_active(),
        }
    }
}
