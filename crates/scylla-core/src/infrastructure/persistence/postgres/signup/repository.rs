use crate::application::authz::grant::Grant;
use crate::application::signup::repository::SignupRepository;
use crate::domain::entities::{Organization, User};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use super::super::error::SqlxResultExt;
use super::super::{grants, organizations, user_organization, users};

/// Cross-aggregate atomic write for self-service signup. Runs the four inserts
/// in one transaction using the shared `queries` helpers, so a failure at any
/// step (e.g. a username unique violation) rolls the whole account back.
#[derive(Clone)]
pub struct PgSignupRepository {
    pool: PgPool,
}

impl PgSignupRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SignupRepository for PgSignupRepository {
    #[instrument(skip(self, user, organization, grant), fields(user_id = %user.id(), org_id = %organization.id()))]
    async fn provision_account(
        &self,
        user: &User,
        organization: &Organization,
        grant: &Grant,
    ) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;

        users::repository::queries::create(&mut *tx, user).await?;
        organizations::repository::queries::create(&mut *tx, organization).await?;
        user_organization::repository::queries::add_member(&mut *tx, user.id(), organization.id())
            .await?;
        grants::insert(&mut *tx, grant).await?;

        tx.commit().await.to_domain()?;
        Ok(())
    }

    #[cfg(feature = "oauth-github")]
    #[instrument(skip(self, user, organization, grant), fields(user_id = %user.id(), org_id = %organization.id(), provider))]
    async fn provision_account_with_identity(
        &self,
        user: &User,
        organization: &Organization,
        grant: &Grant,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;

        users::repository::queries::create(&mut *tx, user).await?;
        organizations::repository::queries::create(&mut *tx, organization).await?;
        user_organization::repository::queries::add_member(&mut *tx, user.id(), organization.id())
            .await?;
        grants::insert(&mut *tx, grant).await?;
        super::super::oauth_identities::repository::queries::link(
            &mut *tx,
            user.id(),
            provider,
            provider_user_id,
        )
        .await?;

        tx.commit().await.to_domain()?;
        Ok(())
    }
}
