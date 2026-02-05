use crate::persistence::surrealdb::id_mapper::ToRecordId;
use domain::entities::{Session, SessionId, UserId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::SessionRepository;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

pub struct SurrealSessionRepository {
    db: Arc<Surreal<Any>>,
}

impl SurrealSessionRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }
}

impl SessionRepository for SurrealSessionRepository {
    fn create(&self, session: &Session) -> impl Future<Output = DomainResult<Session>> + Send {
        let db = self.db.clone();
        let session = session.clone();
        async move {
            let created: Option<Session> = db
                .create(session.id().to_record_id())
                .content(session.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            created.ok_or_else(|| DomainError::infrastructure("Failed to create session"))
        }
    }

    fn find_by_token(&self, token: &str) -> impl Future<Output = DomainResult<Session>> + Send {
        let db = self.db.clone();
        let token = token.to_string();
        let table = SessionId::table_name().to_string();
        async move {
            let mut results: Vec<Session> = db
                .query("SELECT * FROM type::table($table) WHERE token = $token LIMIT 1")
                .bind(("table", table))
                .bind(("token", token.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            results
                .pop()
                .ok_or_else(|| DomainError::not_found("Session", token))
        }
    }

    fn update(&self, session: &Session) -> impl Future<Output = DomainResult<Session>> + Send {
        let db = self.db.clone();
        let session = session.clone();
        async move {
            let updated: Option<Session> = db
                .update(session.id().to_record_id())
                .content(session.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            updated.ok_or_else(|| DomainError::not_found("Session", session.id().to_string()))
        }
    }

    fn delete_by_token(&self, token: &str) -> impl Future<Output = DomainResult<()>> + Send {
        let db = self.db.clone();
        let token = token.to_string();
        let table = SessionId::table_name().to_string();
        async move {
            db.query("DELETE FROM type::table($table) WHERE token = $token")
                .bind(("table", table))
                .bind(("token", token))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(())
        }
    }

    fn delete_all_for_user(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = DomainResult<u64>> + Send {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();
        let table = SessionId::table_name().to_string();
        async move {
            // First count how many we'll delete
            let count_result: Vec<serde_json::Value> = db
                .query("SELECT count() FROM type::table($table) WHERE user_id = $user_id GROUP ALL")
                .bind(("table", table.clone()))
                .bind(("user_id", user_id_str.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let count = count_result
                .first()
                .and_then(|v| v.get("count"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            // Then delete
            db.query("DELETE FROM type::table($table) WHERE user_id = $user_id")
                .bind(("table", table))
                .bind(("user_id", user_id_str))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(count)
        }
    }

    fn delete_expired(&self) -> impl Future<Output = DomainResult<u64>> + Send {
        let db = self.db.clone();
        let table = SessionId::table_name().to_string();
        async move {
            let now = chrono::Utc::now().to_rfc3339();

            // First count how many we'll delete
            let count_result: Vec<serde_json::Value> = db
                .query("SELECT count() FROM type::table($table) WHERE expires_at < $now GROUP ALL")
                .bind(("table", table.clone()))
                .bind(("now", now.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let count = count_result
                .first()
                .and_then(|v| v.get("count"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            // Then delete
            db.query("DELETE FROM type::table($table) WHERE expires_at < $now")
                .bind(("table", table))
                .bind(("now", now))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(count)
        }
    }

    fn list_for_user(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = DomainResult<Vec<Session>>> + Send {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();
        let table = SessionId::table_name().to_string();
        async move {
            let sessions: Vec<Session> = db
                .query("SELECT * FROM type::table($table) WHERE user_id = $user_id ORDER BY created_at DESC")
                .bind(("table", table))
                .bind(("user_id", user_id_str))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            Ok(sessions)
        }
    }
}
