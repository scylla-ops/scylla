use crate::persistence::surrealdb::id_mapper::ToRecordId;
use domain::entities::{User, UserId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::UserRepository;
use domain::value_objects::user::UserName;
use domain::value_objects::{PaginatedResult, PaginationParams};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

pub struct SurrealUserRepository {
    db: Arc<Surreal<Any>>,
}

impl SurrealUserRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }
}

impl UserRepository for SurrealUserRepository {
    fn create(&self, user: &User) -> impl Future<Output = DomainResult<User>> + Send {
        let db = self.db.clone();
        let user = user.clone();
        async move {
            let created: Option<User> = db
                .create(user.id().to_record_id())
                .content(user.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            created.ok_or_else(|| DomainError::infrastructure("Failed to create user"))
        }
    }

    fn find_by_id(&self, id: &UserId) -> impl Future<Output = DomainResult<User>> + Send {
        let db = self.db.clone();
        let id = id.clone();
        async move {
            let result: Option<User> = db
                .select(id.to_record_id())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            result.ok_or_else(|| DomainError::not_found("User", id.to_string()))
        }
    }

    fn find_by_username(
        &self,
        username: &UserName,
    ) -> impl Future<Output = DomainResult<User>> + Send {
        let db = self.db.clone();
        let username_str = username.to_string();
        let table = UserId::table_name().to_string();
        async move {
            let mut results: Vec<User> = db
                .query("SELECT * FROM type::table($table) WHERE username = $username LIMIT 1")
                .bind(("table", table))
                .bind(("username", username_str.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            results
                .pop()
                .ok_or_else(|| DomainError::not_found("User", username_str))
        }
    }

    fn update(&self, user: &User) -> impl Future<Output = DomainResult<User>> + Send {
        let db = self.db.clone();
        let user = user.clone();
        async move {
            let updated: Option<User> = db
                .update(user.id().to_record_id())
                .content(user.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            updated.ok_or_else(|| DomainError::not_found("User", user.id().to_string()))
        }
    }

    fn delete(&self, id: &UserId) -> impl Future<Output = DomainResult<()>> + Send {
        let db = self.db.clone();
        let id = id.clone();
        async move {
            db.delete::<Option<User>>(id.to_record_id())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(())
        }
    }

    fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<User>>> + Send {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserId::table_name().to_string();
        async move {
            // Get total count
            let count_result: Vec<serde_json::Value> = db
                .query("SELECT count() FROM type::table($table) GROUP ALL")
                .bind(("table", table.clone()))
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
            let users: Vec<User> = db
                .query("SELECT * FROM type::table($table) ORDER BY created_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            Ok(PaginatedResult::new(users, &params, total_count))
        }
    }

    fn username_exists(
        &self,
        username: &UserName,
    ) -> impl Future<Output = DomainResult<bool>> + Send {
        let db = self.db.clone();
        let username_str = username.to_string();
        let table = UserId::table_name().to_string();
        async move {
            let count_result: Vec<serde_json::Value> = db
                .query(
                    "SELECT count() FROM type::table($table) WHERE username = $username GROUP ALL",
                )
                .bind(("table", table))
                .bind(("username", username_str))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let count = count_result
                .first()
                .and_then(|v| v.get("count"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            Ok(count > 0)
        }
    }
}
