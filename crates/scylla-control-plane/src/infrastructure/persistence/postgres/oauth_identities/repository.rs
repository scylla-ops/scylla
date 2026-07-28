use crate::application::oauth::OAuthIdentityRepository;
use crate::domain::errors::DomainResult;
use crate::domain::ids::UserId;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::SqlxResultExt;

#[derive(Clone)]
pub struct PgOAuthIdentityRepository {
    pool: PgPool,
}

impl PgOAuthIdentityRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OAuthIdentityRepository for PgOAuthIdentityRepository {
    #[instrument(skip(self), fields(provider, provider_user_id))]
    async fn find_user_id(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<Option<UserId>> {
        queries::find_user_id(&self.pool, provider, provider_user_id).await
    }

    #[instrument(skip(self), fields(user_id = %user_id, provider))]
    async fn link(
        &self,
        user_id: &UserId,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<()> {
        queries::link(&self.pool, user_id, provider, provider_user_id).await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    pub async fn find_user_id<'e, E>(
        executor: E,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<Option<UserId>>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT user_id
            FROM user_oauth_identities
            WHERE provider = $1 AND provider_user_id = $2
            "#,
            provider,
            provider_user_id,
        )
        .fetch_optional(executor)
        .await
        .to_domain()?;
        Ok(rec.map(|r| UserId::new(r.user_id)))
    }

    pub async fn link<'e, E>(
        executor: E,
        user_id: &UserId,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO user_oauth_identities (user_id, provider, provider_user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (provider, provider_user_id) DO NOTHING
            "#,
            user_id.as_str(),
            provider,
            provider_user_id,
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }
}
