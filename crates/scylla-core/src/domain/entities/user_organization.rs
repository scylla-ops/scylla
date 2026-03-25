use crate::domain::entities::{OrganizationId, UserId, UserOrganizationId};

#[derive(Debug, Clone)]
pub struct UserOrganization {
    id: UserOrganizationId,
    user_id: UserId,
    organization_id: OrganizationId,
}

impl UserOrganization {
    #[must_use] 
    pub fn new(id: UserOrganizationId, user_id: UserId, organization_id: OrganizationId) -> Self {
        Self {
            id,
            user_id,
            organization_id,
        }
    }

    #[must_use] 
    pub fn id(&self) -> &UserOrganizationId {
        &self.id
    }

    #[must_use] 
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    #[must_use] 
    pub fn organization_id(&self) -> &OrganizationId {
        &self.organization_id
    }
}
