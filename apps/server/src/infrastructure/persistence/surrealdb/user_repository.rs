use crate::domain::entities::User;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::UserRepository;
use crate::domain::value_objects::{UserId, Username};
use crate::infrastructure::persistence::mappers::ToRecordId;
use crate::infrastructure::persistence::surrealdb::models::UserRecord;
use crate::infrastructure::persistence::{UserInsert, UserUpdate};
use async_trait::async_trait;
use derive_more::Constructor;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// SurrealDB implementation of UserRepository
#[derive(Constructor)]
pub struct SurrealUserRepository {
    db: Arc<Surreal<Any>>,
}

#[async_trait]
impl UserRepository for SurrealUserRepository {
    async fn create(&self, user: &User) -> DomainResult<User> {
        let insert = UserInsert::from(user);
        let created: Option<UserRecord> = self
            .db
            .create(user.id().to_record_id())
            .content(insert)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match created {
            Some(record) => Ok(User::try_from(record)?),
            None => Err(DomainError::infrastructure("Failed to create user")),
        }
    }
    async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
        let result: Option<UserRecord> = self
            .db
            .select(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match result {
            Some(record) => Ok(User::try_from(record)?),
            None => Err(DomainError::not_found("User", id.to_string())),
        }
    }

    async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
        let mut results: Vec<UserRecord> = self
            .db
            .query("SELECT * FROM type::table($table) WHERE username = $username LIMIT 1")
            .bind(("table", UserId::table_name()))
            .bind(("username", username.to_string()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        match results.pop() {
            Some(record) => Ok(User::try_from(record)?),
            None => Err(DomainError::not_found("User", username.to_string())),
        }
    }

    async fn update(&self, user: &User) -> DomainResult<User> {
        let record = UserUpdate::from(user);
        let updated: Option<UserRecord> = self
            .db
            .update(user.id().to_record_id())
            .merge(record)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match updated {
            Some(record) => Ok(User::try_from(record)?),
            None => Err(DomainError::not_found("User", user.id().to_string())),
        }
    }

    async fn delete(&self, id: &UserId) -> DomainResult<()> {
        self.db
            .delete::<Option<UserRecord>>(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(
        &self,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<User>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        // Use provided pagination or default
        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM type::table($table) GROUP ALL")
            .bind(("table", UserId::table_name()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result
            .first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Get paginated records
        let records: Vec<UserRecord> = self
            .db
            .query("SELECT * FROM type::table($table) ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("table", UserId::table_name()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let users: DomainResult<Vec<User>> = records
            .into_iter()
            .map(|record| User::try_from(record))
            .collect();

        Ok(PaginatedResult::new(users?, &params, total_count))
    }

    async fn username_exists(&self, username: &Username) -> DomainResult<bool> {
        match self.find_by_username(username).await {
            Ok(_) => Ok(true),
            Err(DomainError::NotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
