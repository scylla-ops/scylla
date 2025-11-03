use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{
    OrganizationId, UserId, UserOrganizationId, UserOrganizationRole,
};
use chrono::{DateTime, Utc};
use derive_more::Constructor;

/// UserOrganization domain entity
#[derive(Debug, Clone, Constructor)]
pub struct UserOrganization {
    id: UserOrganizationId,
    user_id: UserId,
    organization_id: OrganizationId,
    role: UserOrganizationRole,
    joined_at: DateTime<Utc>,
}

impl UserOrganization {
    /// Create a new organization
    pub fn create(
        user_id: UserId,
        organization_id: OrganizationId,
        role: UserOrganizationRole,
    ) -> DomainResult<Self> {
        let now = Utc::now();
        Ok(Self {
            id: UserOrganizationId::generate(),
            user_id,
            organization_id,
            role,
            joined_at: now,
        })
    }

    // Getters
    pub fn id(&self) -> &UserOrganizationId {
        &self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn organization_id(&self) -> &OrganizationId {
        &self.organization_id
    }

    pub fn role(&self) -> &UserOrganizationRole {
        &self.role
    }

    pub fn joined_at(&self) -> DateTime<Utc> {
        self.joined_at
    }
}
