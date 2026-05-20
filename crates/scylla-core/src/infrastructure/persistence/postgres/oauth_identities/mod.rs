use crate::application::oauth::OAuthIdentityRepository;
use crate::domain::entities::UserId;
use crate::domain::errors::{DomainError, DomainResult};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use tracing::instrument;

#[cfg(test)]
mod tests;

/// Persistence for OAuth identity links (`user_oauth_identities`). Runtime
/// queries (like grants/invitations) so the `oauth-github`-gated SQL needs no
/// offline cache entries.
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
        let row = sqlx::query(
            "SELECT user_id FROM user_oauth_identities WHERE provider = $1 AND provider_user_id = $2",
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(infra)?;
        row.map(|r| r.try_get::<String, _>("user_id").map(UserId::new).map_err(infra))
            .transpose()
    }

    #[instrument(skip(self), fields(user_id = %user_id, provider))]
    async fn link(
        &self,
        user_id: &UserId,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO user_oauth_identities (user_id, provider, provider_user_id) \
             VALUES ($1, $2, $3) ON CONFLICT (provider, provider_user_id) DO NOTHING",
        )
        .bind(user_id.as_str())
        .bind(provider)
        .bind(provider_user_id)
        .execute(&self.pool)
        .await
        .map_err(infra)?;
        Ok(())
    }
}

fn infra<E: std::fmt::Display>(e: E) -> DomainError {
    DomainError::Infrastructure(e.to_string())
}
