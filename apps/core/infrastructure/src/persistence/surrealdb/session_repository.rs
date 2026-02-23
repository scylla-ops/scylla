use async_trait::async_trait;
use domain::entities::{Session, SessionId, UserId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::SessionRepository;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb::Surreal;

#[derive(Clone)]
pub struct SurrealSessionRepository {
    db: Surreal<Any>,
}

impl SurrealSessionRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SessionRepository for SurrealSessionRepository {
    async fn create(&self, session: &Session) -> DomainResult<Session> {
        let db = self.db.clone();
        let session = session.clone();
        let created: Option<Session> = db
            .create(RecordId::new(
                SessionId::table_name(),
                session.id().as_str(),
            ))
            .content(session.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        created.ok_or_else(|| DomainError::infrastructure("Failed to create session"))
    }

    async fn find_by_token(&self, token: &str) -> DomainResult<Session> {
        let db = self.db.clone();
        let token = token.to_string();
        let table = SessionId::table_name().to_string();
        let mut response = db
            .query("SELECT * FROM type::table($table) WHERE token = $session_token LIMIT 1")
            .bind(("table", table))
            .bind(("session_token", token.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        let mut results: Vec<Session> = response
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        results
            .pop()
            .ok_or_else(|| DomainError::not_found("Session", token))
    }

    async fn update(&self, session: &Session) -> DomainResult<Session> {
        let db = self.db.clone();
        let session = session.clone();
        let updated: Option<Session> = db
            .update(RecordId::new(
                SessionId::table_name(),
                session.id().as_str(),
            ))
            .content(session.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        updated.ok_or_else(|| DomainError::not_found("Session", session.id().to_string()))
    }

    async fn delete_by_token(&self, token: &str) -> DomainResult<()> {
        let db = self.db.clone();
        let token = token.to_string();
        let table = SessionId::table_name().to_string();
        db.query("DELETE FROM type::table($table) WHERE token = $session_token")
            .bind(("table", table))
            .bind(("session_token", token))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn delete_expired(&self) -> DomainResult<u64> {
        let db = self.db.clone();
        let table = SessionId::table_name().to_string();

        let now = chrono::Utc::now().to_rfc3339();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE expires_at < $now GROUP ALL")
            .bind(("table", table.clone()))
            .bind(("now", now.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let count = count_result.first().copied().unwrap_or(0) as u64;

        db.query("DELETE FROM type::table($table) WHERE expires_at < $now")
            .bind(("table", table))
            .bind(("now", now))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(count)
    }

    async fn list_for_user(&self, user_id: &UserId) -> DomainResult<Vec<Session>> {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();
        let table = SessionId::table_name().to_string();
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
