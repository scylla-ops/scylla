use crate::application::invitation::{Invitation, InvitationRepository, InvitationStatus};
use crate::application::permission::grant::Grant;
use crate::domain::entities::{InvitationId, OrganizationId, User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::role::name::RoleName;
use crate::domain::value_objects::user::Email;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tracing::instrument;

use super::error::SqlxResultExt;
use super::{grants, user_organization, users};

#[cfg(test)]
mod tests;

/// Persistence for organization invitations. Uses runtime `sqlx::query` (like
/// grants) so the `invitations`-gated SQL needs no offline cache entries.
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

fn row_into_invitation(row: &sqlx::postgres::PgRow) -> DomainResult<Invitation> {
    let role_name: Option<String> = row.try_get("role_name").map_err(infra)?;
    let role = role_name.map(RoleName::new).transpose()?;
    let email: String = row.try_get("email").map_err(infra)?;
    let status: String = row.try_get("status").map_err(infra)?;
    Ok(Invitation {
        id: InvitationId::new(row.try_get::<String, _>("id").map_err(infra)?),
        organization_id: OrganizationId::new(
            row.try_get::<String, _>("organization_id").map_err(infra)?,
        ),
        email: Email::new(email)?,
        role,
        token: row.try_get("token").map_err(infra)?,
        status: InvitationStatus::parse(&status)?,
        invited_by: UserId::new(row.try_get::<String, _>("invited_by").map_err(infra)?),
        expires_at: row.try_get::<DateTime<Utc>, _>("expires_at").map_err(infra)?,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at").map_err(infra)?,
    })
}

const SELECT_COLS: &str =
    "id, organization_id, email, role_name, token, status, invited_by, expires_at, created_at";

#[async_trait]
impl InvitationRepository for PgInvitationRepository {
    #[instrument(skip(self, invite), fields(invite_id = %invite.id))]
    async fn create(&self, invite: &Invitation) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO organization_invites \
             (id, organization_id, email, role_name, token, status, invited_by, expires_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(invite.id.as_str())
        .bind(invite.organization_id.as_str())
        .bind(invite.email.as_str())
        .bind(invite.role.as_ref().map(RoleName::as_str))
        .bind(&invite.token)
        .bind(invite.status.as_str())
        .bind(invite.invited_by.as_str())
        .bind(invite.expires_at)
        .bind(invite.created_at)
        .execute(&self.pool)
        .await
        .map_err(infra)?;
        Ok(())
    }

    #[instrument(skip(self), fields(invite_id = %id))]
    async fn find_by_id(&self, id: &InvitationId) -> DomainResult<Invitation> {
        let row = sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM organization_invites WHERE id = $1"
        ))
        .bind(id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(infra)?
        .ok_or_else(|| DomainError::not_found("Invitation", id.to_string()))?;
        row_into_invitation(&row)
    }

    #[instrument(skip(self, token))]
    async fn find_by_token(&self, token: &str) -> DomainResult<Invitation> {
        let row = sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM organization_invites WHERE token = $1"
        ))
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(infra)?
        .ok_or_else(|| DomainError::not_found("Invitation", "<token>"))?;
        row_into_invitation(&row)
    }

    #[instrument(skip(self), fields(org_id = %org_id))]
    async fn list_pending(&self, org_id: &OrganizationId) -> DomainResult<Vec<Invitation>> {
        let rows = sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM organization_invites \
             WHERE organization_id = $1 AND status = 'pending' ORDER BY created_at DESC"
        ))
        .bind(org_id.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        rows.iter().map(row_into_invitation).collect()
    }

    #[instrument(skip(self), fields(invite_id = %id))]
    async fn revoke(&self, id: &InvitationId) -> DomainResult<()> {
        sqlx::query("UPDATE organization_invites SET status = 'revoked' WHERE id = $1")
            .bind(id.as_str())
            .execute(&self.pool)
            .await
            .map_err(infra)?;
        Ok(())
    }

    #[instrument(skip(self, new_user, grant), fields(invite_id = %invite_id, member = %member))]
    async fn accept_atomic(
        &self,
        invite_id: &InvitationId,
        new_user: Option<&User>,
        member: &UserId,
        organization_id: &OrganizationId,
        grant: Option<&Grant>,
    ) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;

        if let Some(user) = new_user {
            users::repository::queries::create(&mut *tx, user).await?;
        }
        user_organization::repository::queries::add_member(&mut *tx, member, organization_id)
            .await?;
        if let Some(grant) = grant {
            grants::insert(&mut *tx, grant).await?;
        }
        sqlx::query("UPDATE organization_invites SET status = 'accepted' WHERE id = $1")
            .bind(invite_id.as_str())
            .execute(&mut *tx)
            .await
            .map_err(infra)?;

        tx.commit().await.to_domain()?;
        Ok(())
    }
}

fn infra<E: std::fmt::Display>(e: E) -> DomainError {
    DomainError::Infrastructure(e.to_string())
}
