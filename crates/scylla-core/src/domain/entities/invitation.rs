use crate::domain::clock;
use crate::domain::entities::ids::{InvitationId, OrganizationId, UserId};
use crate::domain::value_objects::invitation::InvitationStatus;
use crate::domain::value_objects::role::name::RoleName;
use crate::domain::value_objects::user::Email;
use chrono::{DateTime, Duration, Utc};

const INVITE_TTL_DAYS: i64 = 7;

/// An email-based invitation to join an organization, optionally with a scoped
/// role granted on acceptance.
#[derive(Debug, Clone)]
pub struct Invitation {
    id: InvitationId,
    organization_id: OrganizationId,
    email: Email,
    role: Option<RoleName>,
    token: String,
    status: InvitationStatus,
    invited_by: UserId,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl Invitation {
    #[must_use]
    pub fn create(
        organization_id: OrganizationId,
        email: Email,
        role: Option<RoleName>,
        invited_by: UserId,
    ) -> Self {
        let now = clock::now();
        Self {
            id: InvitationId::generate(),
            organization_id,
            email,
            role,
            token: uuid::Uuid::new_v4().to_string(),
            status: InvitationStatus::Pending,
            invited_by,
            expires_at: now + Duration::days(INVITE_TTL_DAYS),
            created_at: now,
        }
    }

    /// Rehydrate an invitation from persisted columns.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn from_persistence(
        id: InvitationId,
        organization_id: OrganizationId,
        email: Email,
        role: Option<RoleName>,
        token: String,
        status: InvitationStatus,
        invited_by: UserId,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            organization_id,
            email,
            role,
            token,
            status,
            invited_by,
            expires_at,
            created_at,
        }
    }

    #[must_use]
    pub fn is_acceptable(&self) -> bool {
        self.status == InvitationStatus::Pending && clock::now() <= self.expires_at
    }

    #[must_use]
    pub fn id(&self) -> &InvitationId {
        &self.id
    }

    #[must_use]
    pub fn organization_id(&self) -> &OrganizationId {
        &self.organization_id
    }

    #[must_use]
    pub fn email(&self) -> &Email {
        &self.email
    }

    #[must_use]
    pub fn role(&self) -> Option<&RoleName> {
        self.role.as_ref()
    }

    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[must_use]
    pub fn status(&self) -> InvitationStatus {
        self.status
    }

    #[must_use]
    pub fn invited_by(&self) -> &UserId {
        &self.invited_by
    }

    #[must_use]
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
