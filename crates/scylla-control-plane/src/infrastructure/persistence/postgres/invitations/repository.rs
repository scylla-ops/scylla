use crate::application::authz::grant::Grant;
use crate::application::invitation::InvitationRepository;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{InvitationId, OrganizationId, UserId};
use crate::domain::invitation::Invitation;
use crate::domain::user::User;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::SqlxResultExt;
use super::super::{grants, users};

#[derive(Clone)]
pub struct PgInvitationRepository {
    pool: PgPool,
}

impl PgInvitationRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InvitationRepository for PgInvitationRepository {
    #[instrument(skip(self, invite), fields(invite_id = %invite.id()))]
    async fn create(&self, invite: &Invitation) -> DomainResult<()> {
        queries::create(&self.pool, invite).await
    }

    #[instrument(skip(self), fields(invite_id = %id))]
    async fn find_by_id(&self, id: &InvitationId) -> DomainResult<Invitation> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip(self, token))]
    async fn find_by_token(&self, token: &str) -> DomainResult<Invitation> {
        queries::find_by_token(&self.pool, token).await
    }

    #[instrument(skip(self), fields(org_id = %org_id))]
    async fn list_pending(&self, org_id: &OrganizationId) -> DomainResult<Vec<Invitation>> {
        queries::list_pending(&self.pool, org_id).await
    }

    #[instrument(skip(self), fields(invite_id = %id))]
    async fn revoke(&self, id: &InvitationId) -> DomainResult<()> {
        queries::revoke(&self.pool, id).await
    }

    #[instrument(skip(self, new_user, grant), fields(invite_id = %invite_id, member = %member))]
    async fn accept_atomic(
        &self,
        invite_id: &InvitationId,
        new_user: Option<&User>,
        member: &UserId,
        organization_id: &OrganizationId,
        grant: &Grant,
    ) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;

        if let Some(user) = new_user {
            users::repository::queries::create(&mut *tx, user).await?;
        }
        grants::insert(&mut *tx, grant).await?;
        queries::mark_accepted(&mut *tx, invite_id).await?;

        tx.commit().await.to_domain()?;
        Ok(())
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;
    use crate::domain::invitation::InvitationStatus;
    use crate::domain::role::RoleName;
    use crate::domain::user::Email;
    use crate::infrastructure::persistence::postgres::error::DbFieldExt;
    use chrono::{DateTime, Utc};

    #[allow(clippy::too_many_arguments)]
    fn row_into_invitation(
        id: String,
        organization_id: String,
        email: String,
        role_name: Option<String>,
        token: String,
        status: String,
        invited_by: String,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> DomainResult<Invitation> {
        let email = Email::new(email).db_field("email")?;
        let role = role_name
            .map(RoleName::new)
            .transpose()
            .db_field("role name")?;
        let status = InvitationStatus::new(status).db_field("invitation status")?;
        Ok(Invitation::from_persistence(
            InvitationId::new(id),
            OrganizationId::new(organization_id),
            email,
            role,
            token,
            status,
            UserId::new(invited_by),
            expires_at,
            created_at,
        ))
    }

    pub async fn create<'e, E>(executor: E, invite: &Invitation) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO organization_invites
                (id, organization_id, email, role_name, token, status, invited_by, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            invite.id().as_str(),
            invite.organization_id().as_str(),
            invite.email().as_str(),
            invite.role().map(RoleName::as_str),
            invite.token(),
            invite.status().as_str(),
            invite.invited_by().as_str(),
            invite.expires_at(),
            invite.created_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &InvitationId) -> DomainResult<Invitation>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, organization_id, email, role_name, token, status, invited_by, expires_at, created_at
            FROM organization_invites
            WHERE id = $1
            "#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("Invitation", id.to_string())?;
        row_into_invitation(
            rec.id,
            rec.organization_id,
            rec.email,
            rec.role_name,
            rec.token,
            rec.status,
            rec.invited_by,
            rec.expires_at,
            rec.created_at,
        )
    }

    pub async fn find_by_token<'e, E>(executor: E, token: &str) -> DomainResult<Invitation>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, organization_id, email, role_name, token, status, invited_by, expires_at, created_at
            FROM organization_invites
            WHERE token = $1
            "#,
            token,
        )
        .fetch_one(executor)
        .await
        .not_found_as("Invitation", "<token>")?;
        row_into_invitation(
            rec.id,
            rec.organization_id,
            rec.email,
            rec.role_name,
            rec.token,
            rec.status,
            rec.invited_by,
            rec.expires_at,
            rec.created_at,
        )
    }

    pub async fn list_pending<'e, E>(
        executor: E,
        org_id: &OrganizationId,
    ) -> DomainResult<Vec<Invitation>>
    where
        E: PgExecutor<'e>,
    {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, email, role_name, token, status, invited_by, expires_at, created_at
            FROM organization_invites
            WHERE organization_id = $1 AND status = 'pending'
            ORDER BY created_at DESC
            "#,
            org_id.as_str(),
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter()
            .map(|r| {
                row_into_invitation(
                    r.id,
                    r.organization_id,
                    r.email,
                    r.role_name,
                    r.token,
                    r.status,
                    r.invited_by,
                    r.expires_at,
                    r.created_at,
                )
            })
            .collect()
    }

    pub async fn revoke<'e, E>(executor: E, id: &InvitationId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            "UPDATE organization_invites SET status = 'revoked' WHERE id = $1",
            id.as_str(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }

    pub async fn mark_accepted<'e, E>(executor: E, id: &InvitationId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            "UPDATE organization_invites SET status = 'accepted' WHERE id = $1",
            id.as_str(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }
}
