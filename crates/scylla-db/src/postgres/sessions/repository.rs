use crate::domain::errors::DomainResult;
use crate::domain::ids::{SessionId, UserId};
use crate::domain::session::Session;
use async_trait::async_trait;
use scylla_core::application::SessionRepository;
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::SqlxResultExt;

#[derive(Clone)]
pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    #[instrument(skip_all, fields(session_id = %session.id()))]
    async fn create(&self, session: &Session) -> DomainResult<Session> {
        queries::create(&self.pool, session).await
    }

    #[instrument(skip(self, token))]
    async fn find_by_token(&self, token: &str) -> DomainResult<Session> {
        queries::find_by_token(&self.pool, token).await
    }

    #[instrument(skip(self, token))]
    async fn delete_by_token(&self, token: &str) -> DomainResult<()> {
        queries::delete_by_token(&self.pool, token).await
    }

    #[instrument(skip(self))]
    async fn delete_expired(&self) -> DomainResult<u64> {
        queries::delete_expired(&self.pool).await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    pub async fn create<'e, E>(executor: E, session: &Session) -> DomainResult<Session>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO sessions (id, token, user_id, created_at, expires_at, last_active_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            session.id().as_str(),
            session.token(),
            session.user_id().as_str(),
            session.created_at(),
            session.expires_at(),
            session.last_active_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(session.clone())
    }

    pub async fn find_by_token<'e, E>(executor: E, token: &str) -> DomainResult<Session>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, token, user_id, created_at, expires_at, last_active_at
            FROM sessions
            WHERE token = $1
            "#,
            token,
        )
        .fetch_one(executor)
        .await
        // Never echo the session token into the error id.
        .not_found_as("Session", "<token>")?;
        Ok(Session::from_persistence(
            SessionId::new(rec.id),
            rec.token,
            UserId::new(rec.user_id),
            rec.created_at,
            rec.expires_at,
            rec.last_active_at,
        ))
    }

    pub async fn delete_by_token<'e, E>(executor: E, token: &str) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!("DELETE FROM sessions WHERE token = $1", token)
            .execute(executor)
            .await
            .to_domain()?;
        Ok(())
    }

    pub async fn delete_expired<'e, E>(executor: E) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let res = sqlx::query!("DELETE FROM sessions WHERE expires_at <= NOW()")
            .execute(executor)
            .await
            .to_domain()?;
        Ok(res.rows_affected())
    }
}
