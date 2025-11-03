use crate::domain::entities::Project;
use crate::domain::value_objects::{Description, OrganizationId, ProjectId, ProjectName};
use crate::domain::{DomainError, DomainResult};
use crate::infrastructure::persistence::ProjectUpdate;
use crate::infrastructure::persistence::mappers::ToRecordId;
use crate::infrastructure::persistence::surrealdb::models::{ProjectInsert, ProjectRecord};
use chrono::DateTime;
use std::convert::{From, TryFrom};

impl TryFrom<ProjectRecord> for Project {
    type Error = DomainError;

    /// Convert database record to domain entity
    fn try_from(record: ProjectRecord) -> DomainResult<Self> {
        let id = ProjectId::new(record.id.key().to_string());
        let organization_id = OrganizationId::new(record.organization.key().to_string());
        let name = ProjectName::new(record.name)?;
        let description = record.description.map(Description::new).transpose()?;

        Ok(Project::new(
            id,
            name,
            description,
            organization_id,
            record.is_active,
            DateTime::from(record.created_at),
            DateTime::from(record.updated_at),
        ))
    }
}

impl From<&Project> for ProjectInsert {
    /// Convert domain entity to insert record
    fn from(project: &Project) -> Self {
        ProjectInsert {
            name: project.name().as_str().to_string(),
            description: project.description().map(|d| d.as_str().to_string()),
            organization: project.organization_id().to_record_id(),
            is_active: project.is_active(),
        }
    }
}

impl From<&Project> for ProjectUpdate {
    /// Convert domain entity to update record
    fn from(project: &Project) -> Self {
        ProjectUpdate {
            name: project.name().as_str().to_string(),
            description: project.description().map(|d| d.as_str().to_string()),
            is_active: project.is_active(),
        }
    }
}
