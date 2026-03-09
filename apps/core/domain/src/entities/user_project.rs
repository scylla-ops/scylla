use crate::entities::{ProjectId, UserId, UserProjectId};

#[derive(Debug, Clone)]
pub struct UserProject {
    id: UserProjectId,
    user_id: UserId,
    project_id: ProjectId,
}

impl UserProject {
    pub fn new(id: UserProjectId, user_id: UserId, project_id: ProjectId) -> Self {
        Self {
            id,
            user_id,
            project_id,
        }
    }

    pub fn id(&self) -> &UserProjectId {
        &self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }
}
