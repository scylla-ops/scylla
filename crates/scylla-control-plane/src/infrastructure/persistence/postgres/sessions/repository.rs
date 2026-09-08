use crate::application::SessionRepository;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{SessionId, UserId};
use crate::domain::session::Session;
use async_trait::async_trait;
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

    #[instrument(skip_all, fields(session_id = %session.id()))]
    async fn update(&self, session: &Session) -> DomainResult<Session> {
        queries::update(&self.pool, session).await
    }

    #[instrument(skip(self, token))]
    async fn delete_by_token(&self, token: &str) -> DomainResult<()> {
        queries::delete_by_token(&self.pool, token).await
    }

    #[instrument(skip(self))]
    async fn delete_expired(&self) -> DomainResult<u64> {
        queries::delete_expired(&self.pool).await
    }

    #[instrument(skip_all, fields(user_id = %user_id))]
    async fn list_for_user(&self, user_id: &UserId) -> DomainResult<Vec<Session>> {
        queries::list_for_user(&self.pool, user_id).await
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
        // Never echo the raw session token into the error id (logs / tracing /
        // status). Use a non-secret placeholder.
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

    pub async fn update<'e, E>(executor: E, session: &Session) -> DomainResult<Session>
    where
        E: PgExecutor<'e>,
    {
        let res = sqlx::query!(
            r#"
            UPDATE sessions
            SET token = $2,
                expires_at = $3,
                last_active_at = $4
            WHERE id = $1
            "#,
            session.id().as_str(),
            session.token(),
            session.expires_at(),
            session.last_active_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        if res.rows_affected() == 0 {
            return Err(DomainError::not_found("Session", session.id().to_string()));
        }
        Ok(session.clone())
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

    pub async fn list_for_user<'e, E>(executor: E, user_id: &UserId) -> DomainResult<Vec<Session>>
    where
        E: PgExecutor<'e>,
    {
        let rows = sqlx::query!(
            r#"
            SELECT id, token, user_id, created_at, expires_at, last_active_at
            FROM sessions
            WHERE user_id = $1
            ORDER BY last_active_at DESC
            "#,
            user_id.as_str(),
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        Ok(rows
            .into_iter()
            .map(|r| {
                Session::from_persistence(
                    SessionId::new(r.id),
                    r.token,
                    UserId::new(r.user_id),
                    r.created_at,
                    r.expires_at,
                    r.last_active_at,
                )
            })
            .collect())
    }
}
