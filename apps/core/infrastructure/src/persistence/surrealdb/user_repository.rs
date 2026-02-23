use async_trait::async_trait;
use domain::entities::{User, UserId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::UserRepository;
use domain::value_objects::user::UserName;
use domain::value_objects::{PaginatedResult, PaginationParams};
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb::Surreal;
use surrealdb_types::SurrealValue;

pub struct SurrealUserRepository {
    db: Surreal<Any>,
}

impl SurrealUserRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for SurrealUserRepository {
    async fn create(&self, user: &User) -> DomainResult<User> {
        let db = self.db.clone();
        let user = user.clone();
        let created: Option<User> = db
            .create(RecordId::new(UserId::table_name(), user.id().as_str()))
            .content(user.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        created.ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
    }

    async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
        let db = self.db.clone();
        let id = id.clone();
        let result: Option<User> = db
            .select(RecordId::new(UserId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        result.ok_or_else(|| DomainError::not_found("User", id.to_string()))
    }

    async fn find_by_username(&self, username: &UserName) -> DomainResult<User> {
        let db = self.db.clone();
        let table = UserId::table_name().to_string();
        let mut results: Vec<User> = db
            .query("SELECT * FROM type::table($table) WHERE username = $username LIMIT 1")
            .bind(("table", table))
            .bind((
                "username",
                username.clone().into_value().into_object().map_err(|_| {
                    DomainError::infrastructure("Failed to convert username to object".to_string())
                })?,
            ))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        results
            .pop()
            .ok_or_else(|| DomainError::not_found("User", username.as_str()))
    }

    async fn update(&self, user: &User) -> DomainResult<User> {
        let db = self.db.clone();
        let user = user.clone();
        let updated: Option<User> = db
            .update(RecordId::new(UserId::table_name(), user.id().as_str()))
            .content(user.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        updated.ok_or_else(|| DomainError::not_found("User", user.id().to_string()))
    }

    async fn delete(&self, id: &UserId) -> DomainResult<()> {
        let db = self.db.clone();
        let id = id.clone();
        db.delete::<Option<User>>(RecordId::new(UserId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<User>> {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) GROUP ALL")
            .bind(("table", table.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

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

    async fn username_exists(&self, username: &UserName) -> DomainResult<bool> {
        let db = self.db.clone();
        let table = UserId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE username = $username GROUP ALL")
            .bind(("table", table))
            .bind((
                "username",
                username.clone().into_value().into_object().map_err(|_| {
                    DomainError::infrastructure("Failed to convert username to object".to_string())
                })?,
            ))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let count = count_result.first().copied().unwrap_or(0);
        Ok(count > 0)
    }
}
