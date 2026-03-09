use crate::entities::{OrganizationId, UserId, UserOrganizationId};

#[derive(Debug, Clone)]
pub struct UserOrganization {
    id: UserOrganizationId,
    user_id: UserId,
    organization_id: OrganizationId,
}

impl UserOrganization {
    pub fn new(id: UserOrganizationId, user_id: UserId, organization_id: OrganizationId) -> Self {
        Self {
            id,
            user_id,
            organization_id,
        }
    }

    pub fn id(&self) -> &UserOrganizationId {
        &self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn organization_id(&self) -> &OrganizationId {
        &self.organization_id
    }
}
