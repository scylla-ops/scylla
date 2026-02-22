use crate::entities::{OrganizationId, ProjectId};
use crate::errors::{DomainError, DomainResult};
use crate::value_objects::project::{ProjectDescription, ProjectName};
use chrono::{DateTime, Utc};
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

/// Project domain entity
#[derive(Debug, Clone)]
#[cfg_attr(feature = "surrealdb", derive(surrealdb_types::SurrealValue))]
pub struct Project {
    id: ProjectId,
    name: ProjectName,
    description: Option<ProjectDescription>,
    organization_id: OrganizationId,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Project {
    pub fn create(
        name: ProjectName,
        description: Option<ProjectDescription>,
        organization_id: OrganizationId,
    ) -> DomainResult<Self> {
        let now = Utc::now();
        Ok(Self {
            id: ProjectId::generate(),
            name,
            description,
            organization_id,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_name(&mut self, name: ProjectName) -> DomainResult<()> {
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_description(
        &mut self,
        description: Option<ProjectDescription>,
    ) -> DomainResult<()> {
        self.description = description;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn toggle_active(&mut self) -> DomainResult<()> {
        self.is_active = !self.is_active;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn deactivate(&mut self) -> DomainResult<()> {
        if !self.is_active {
            return Err(DomainError::business_rule("Project is already inactive"));
        }
        self.is_active = false;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn activate(&mut self) -> DomainResult<()> {
        if self.is_active {
            return Err(DomainError::business_rule("Project is already active"));
        }
        self.is_active = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    pub fn name(&self) -> &ProjectName {
        &self.name
    }

    pub fn description(&self) -> Option<&ProjectDescription> {
        self.description.as_ref()
    }

    pub fn organization_id(&self) -> &OrganizationId {
        &self.organization_id
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
