use crate::domain::DomainError;
use crate::domain::entities::UserProject;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::UserProjectRole;
use crate::domain::value_objects::{ProjectId, UserId, UserProjectId};
use crate::infrastructure::persistence::UserProjectUpdate;
use crate::infrastructure::persistence::surrealdb::mappers::{FromRecordId, ToRecordId};
use crate::infrastructure::persistence::surrealdb::models::{UserProjectInsert, UserProjectRecord};
use chrono::DateTime;
use std::convert::{From, TryFrom};

impl TryFrom<UserProjectRecord> for UserProject {
    type Error = DomainError;

    /// Convert database record to domain entity
    fn try_from(record: UserProjectRecord) -> DomainResult<Self> {
        let id = UserProjectId::from_record_id(record.id);
        let user_id = UserId::from_record_id(record.user_id);
        let project_id = ProjectId::from_record_id(record.project_id);
        let role = UserProjectRole::new(&record.role)?;

        Ok(UserProject::new(
            id,
            user_id,
            project_id,
            role,
            DateTime::from(record.joined_at),
        ))
    }
}

impl From<&UserProject> for UserProjectInsert {
    /// Convert domain entity to insert record
    fn from(user_project: &UserProject) -> Self {
        UserProjectInsert {
            user_id: user_project.user_id().to_record_id(),
            project_id: user_project.project_id().to_record_id(),
            role: user_project.role().to_string(),
        }
    }
}

impl From<&UserProject> for UserProjectUpdate {
    /// Convert domain entity to update record
    fn from(user_project: &UserProject) -> Self {
        UserProjectUpdate {
            role: user_project.role().to_string(),
        }
    }
}
