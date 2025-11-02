use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{
    OrganizationId, UserId, UserOrganizationId, UserOrganizationRole,
};
use chrono::{DateTime, Utc};

/// UserOrganization domain entity
#[derive(Debug, Clone)]
pub struct UserOrganization {
    id: UserOrganizationId,
    user_id: UserId,
    organization_id: OrganizationId,
    role: UserOrganizationRole,
    joined_at: DateTime<Utc>,
}

impl UserOrganization {
    /// Create a new organization (for reconstruction from database)
    pub fn new(
        id: UserOrganizationId,
        user_id: UserId,
        organization_id: OrganizationId,
        role: UserOrganizationRole,
        joined_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        Ok(Self {
            id,
            user_id,
            organization_id,
            role,
            joined_at,
        })
    }

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
