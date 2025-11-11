use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{ProjectId, UserId, UserProjectId, UserProjectRole};
use chrono::{DateTime, Utc};
use derive_more::Constructor;

/// UserProject domain entity
#[derive(Debug, Clone, Constructor)]
pub struct UserProject {
    id: UserProjectId,
    user_id: UserId,
    project_id: ProjectId,
    role: UserProjectRole,
    joined_at: DateTime<Utc>,
}

impl UserProject {
    /// Create a new project
    pub fn create(
        user_id: UserId,
        project_id: ProjectId,
        role: UserProjectRole,
    ) -> DomainResult<Self> {
        let now = Utc::now();
        Ok(Self {
            id: UserProjectId::generate(),
            user_id,
            project_id,
            role,
            joined_at: now,
        })
    }

    // Getters
    pub fn id(&self) -> &UserProjectId {
        &self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn role(&self) -> &UserProjectRole {
        &self.role
    }

    pub fn joined_at(&self) -> DateTime<Utc> {
        self.joined_at
    }
}
