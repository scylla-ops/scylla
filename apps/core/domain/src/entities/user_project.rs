use crate::entities::{ProjectId, UserId, UserProjectId};
use crate::errors::DomainResult;
use crate::value_objects::user_project::UserProjectRole;
use chrono::{DateTime, Utc};
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

/// UserProject domain entity
#[derive(Debug, Clone)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct UserProject {
    id: UserProjectId,
    user_id: UserId,
    project_id: ProjectId,
    role: UserProjectRole,
    joined_at: DateTime<Utc>,
}

impl UserProject {
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
