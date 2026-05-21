use crate::application::app::token_repository::AppTokenRepository;
use crate::domain::entities::{AppId, AppToken, AppTokenId};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::SqlxResultExt;

#[derive(Clone)]
pub struct PgAppTokenRepository {
    pool: PgPool,
}

impl PgAppTokenRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AppTokenRepository for PgAppTokenRepository {
    #[instrument(skip(self, token), fields(token_id = %token.id()))]
    async fn create(&self, token: &AppToken) -> DomainResult<()> {
        queries::create(&self.pool, token).await
    }

    #[instrument(skip(self, token))]
    async fn find_by_token(&self, token: &str) -> DomainResult<AppToken> {
        queries::find_by_token(&self.pool, token).await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    pub async fn create<'e, E>(executor: E, token: &AppToken) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO app_tokens (id, token, app_id, created_at, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            token.id().as_str(),
            token.token(),
            token.app_id().as_str(),
            token.created_at(),
            token.expires_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }

    pub async fn find_by_token<'e, E>(executor: E, token: &str) -> DomainResult<AppToken>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, token, app_id, created_at, expires_at
            FROM app_tokens
            WHERE token = $1
            "#,
            token,
        )
        .fetch_one(executor)
        .await
        .not_found_as("AppToken", token.to_string())?;
        Ok(AppToken::from_persistence(
            AppTokenId::new(rec.id),
            rec.token,
            AppId::new(rec.app_id),
            rec.created_at,
            rec.expires_at,
        ))
    }
}
