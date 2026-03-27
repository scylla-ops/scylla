use crate::domain::entities::{ProjectId, UserId, UserProjectId};

#[derive(Debug, Clone)]
pub struct UserProject {
    id: UserProjectId,
    user_id: UserId,
    project_id: ProjectId,
}

impl UserProject {
    #[must_use]
    pub fn new(id: UserProjectId, user_id: UserId, project_id: ProjectId) -> Self {
        Self {
            id,
            user_id,
            project_id,
        }
    }

    #[must_use]
    pub fn id(&self) -> &UserProjectId {
        &self.id
    }

    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    #[must_use]
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }
}
