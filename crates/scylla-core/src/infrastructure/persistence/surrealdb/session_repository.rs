use crate::application::ports::SessionRepository;
use crate::domain::entities::{Session, SessionId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use async_trait::async_trait;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;
use tracing::instrument;

#[derive(Clone)]
pub struct SurrealSessionRepository {
    db: Surreal<Any>,
}

impl SurrealSessionRepository {
    #[must_use]
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SessionRepository for SurrealSessionRepository {
    #[instrument(skip(self, session), fields(session_id = %session.id()))]
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
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        created.ok_or_else(|| DomainError::infrastructure("Failed to create session"))
    }

    #[instrument(skip(self, token))]
    async fn find_by_token(&self, token: &str) -> DomainResult<Session> {
        let db = self.db.clone();
        let token = token.to_string();
        let table = SessionId::table_name().to_string();
        let mut response = db
            .query("SELECT * FROM type::table($table) WHERE token = $session_token LIMIT 1")
            .bind(("table", table))
            .bind(("session_token", token.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        let mut results: Vec<Session> = response
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        results
            .pop()
            .ok_or_else(|| DomainError::not_found("Session", token))
    }

    #[instrument(skip(self, session), fields(session_id = %session.id()))]
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
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        updated.ok_or_else(|| DomainError::not_found("Session", session.id().to_string()))
    }

    #[instrument(skip(self, token))]
    async fn delete_by_token(&self, token: &str) -> DomainResult<()> {
        let db = self.db.clone();
        let token = token.to_string();
        let table = SessionId::table_name().to_string();
        db.query("DELETE FROM type::table($table) WHERE token = $session_token")
            .bind(("table", table))
            .bind(("session_token", token))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_expired(&self) -> DomainResult<u64> {
        let db = self.db.clone();
        let table = SessionId::table_name().to_string();

        let now = chrono::Utc::now();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE expires_at < $now GROUP ALL")
            .bind(("table", table.clone()))
            .bind(("now", now))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        db.query("DELETE FROM type::table($table) WHERE expires_at < $now")
            .bind(("table", table))
            .bind(("now", now))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        Ok(count)
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn list_for_user(&self, user_id: &UserId) -> DomainResult<Vec<Session>> {
        let db = self.db.clone();
        let user_record = user_id.clone().into_value();
        let table = SessionId::table_name().to_string();
        let sessions: Vec<Session> = db
                .query("SELECT * FROM type::table($table) WHERE user_id = $user_id ORDER BY created_at DESC")
                .bind(("table", table))
                .bind(("user_id", user_record))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Session, UserId};
    use crate::infrastructure::test_utils::init_db;
    use chrono::Duration;

    async fn setup() -> Surreal<Any> {
        init_db(&[SessionId::table_name()]).await
    }

    fn test_user_id() -> UserId {
        UserId::generate()
    }

    fn create_test_session(user_id: UserId, token: &str, duration: Duration) -> Session {
        Session::create(user_id, token.to_string(), duration)
    }

    #[tokio::test]
    async fn test_create() {
        let db = setup().await;
        let repo = SurrealSessionRepository::new(db);

        let user_id = test_user_id();
        let session = create_test_session(user_id.clone(), "test-token-create", Duration::hours(1));
        let session_id = session.id().clone();

        let created = repo
            .create(&session)
            .await
            .expect("Failed to create session");
        assert_eq!(created.id(), &session_id);
        assert_eq!(created.token(), session.token());
        assert_eq!(created.user_id(), &user_id);
        assert_eq!(created.created_at(), session.created_at());
        assert_eq!(created.expires_at(), session.expires_at());
        assert_eq!(created.last_active_at(), session.last_active_at());
    }

    #[tokio::test]
    async fn test_find_by_token() {
        let db = setup().await;
        let repo = SurrealSessionRepository::new(db);

        let user_id = test_user_id();
        let session = create_test_session(user_id.clone(), "find-by-token", Duration::hours(1));

        repo.create(&session).await.expect("Failed to create");

        let found = repo
            .find_by_token("find-by-token")
            .await
            .expect("Failed to find session by token");
        assert_eq!(found.token(), "find-by-token");
        assert_eq!(found.user_id(), &user_id);
    }

    #[tokio::test]
    async fn test_find_by_token_not_found() {
        let db = setup().await;
        let repo = SurrealSessionRepository::new(db);

        let result = repo.find_by_token("non-existent-token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update() {
        let db = setup().await;
        let repo = SurrealSessionRepository::new(db);

        let user_id = test_user_id();
        let mut session = create_test_session(user_id, "update-token", Duration::hours(1));

        repo.create(&session).await.expect("Failed to create");

        session.extend(Duration::hours(2));

        let updated = repo.update(&session).await.expect("Failed to update");
        assert_eq!(updated.id(), session.id());
        assert_eq!(updated.token(), "update-token");
        // After extend, expires_at and last_active_at should be updated
        assert_eq!(updated.expires_at(), session.expires_at());
        assert_eq!(updated.last_active_at(), session.last_active_at());
    }

    #[tokio::test]
    async fn test_delete_by_token() {
        let db = setup().await;
        let repo = SurrealSessionRepository::new(db);

        let user_id = test_user_id();
        let session = create_test_session(user_id, "delete-token", Duration::hours(1));

        repo.create(&session).await.expect("Failed to create");

        // Verify it exists
        repo.find_by_token("delete-token")
            .await
            .expect("Session should exist before delete");

        repo.delete_by_token("delete-token")
            .await
            .expect("Failed to delete by token");

        let result = repo.find_by_token("delete-token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_by_token_nonexistent() {
        let db = setup().await;
        let repo = SurrealSessionRepository::new(db);

        // Deleting a non-existent token should not error
        let result = repo.delete_by_token("nonexistent-token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_expired() {
        let db = setup().await;
        let repo = SurrealSessionRepository::new(db);

        let user_id = test_user_id();

        // Create an already-expired session (negative duration)
        let expired_session =
            create_test_session(user_id.clone(), "expired-token", Duration::seconds(-10));

        repo.create(&expired_session)
            .await
            .expect("Failed to create expired session");

        // Create a valid session
        let valid_session = create_test_session(user_id, "valid-token", Duration::hours(1));
        repo.create(&valid_session)
            .await
            .expect("Failed to create valid session");

        let deleted_count = repo
            .delete_expired()
            .await
            .expect("Failed to delete expired sessions");
        assert_eq!(deleted_count, 1);

        // The expired session should be gone
        let result = repo.find_by_token("expired-token").await;
        assert!(result.is_err());

        // The valid session should still exist
        let found = repo
            .find_by_token("valid-token")
            .await
            .expect("Valid session should still exist");
        assert_eq!(found.token(), "valid-token");
    }

    #[tokio::test]
    async fn test_delete_expired_none() {
        let db = setup().await;
        let repo = SurrealSessionRepository::new(db);

        let user_id = test_user_id();
        let session = create_test_session(user_id, "still-valid", Duration::hours(1));
        repo.create(&session).await.expect("Failed to create");

        let deleted_count = repo
            .delete_expired()
            .await
            .expect("Failed to delete expired sessions");
        assert_eq!(deleted_count, 0);
    }

    #[tokio::test]
    async fn test_list_for_user() {
        let db = setup().await;
        let repo = SurrealSessionRepository::new(db);

        let user1 = test_user_id();
        let user2 = test_user_id();

        let session1 = create_test_session(user1.clone(), "user1-token-a", Duration::hours(1));
        repo.create(&session1).await.expect("Failed to create s1");

        let session2 = create_test_session(user1.clone(), "user1-token-b", Duration::hours(1));
        repo.create(&session2).await.expect("Failed to create s2");

        let session3 = create_test_session(user2.clone(), "user2-token-a", Duration::hours(1));
        repo.create(&session3).await.expect("Failed to create s3");

        let user1_sessions = repo
            .list_for_user(&user1)
            .await
            .expect("Failed to list sessions for user1");
        assert_eq!(user1_sessions.len(), 2);
        for s in &user1_sessions {
            assert_eq!(s.user_id(), &user1);
        }

        let user2_sessions = repo
            .list_for_user(&user2)
            .await
            .expect("Failed to list sessions for user2");
        assert_eq!(user2_sessions.len(), 1);
        assert_eq!(user2_sessions[0].user_id(), &user2);
    }

    #[tokio::test]
    async fn test_list_for_user_empty() {
        let db = setup().await;
        let repo = SurrealSessionRepository::new(db);

        let user_id = test_user_id();
        let sessions = repo
            .list_for_user(&user_id)
            .await
            .expect("Failed to list sessions");
        assert!(sessions.is_empty());
    }
}
