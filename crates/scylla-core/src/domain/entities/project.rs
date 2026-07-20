use crate::domain::clock;
use crate::domain::entities::{OrganizationId, ProjectId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::project::{ProjectDescription, ProjectName};
use chrono::{DateTime, Utc};

/// Project domain entity
#[derive(Debug, Clone)]
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
    #[must_use]
    pub fn from_persistence(
        id: ProjectId,
        name: ProjectName,
        description: Option<ProjectDescription>,
        organization_id: OrganizationId,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            organization_id,
            is_active,
            created_at,
            updated_at,
        }
    }

    pub fn create(
        name: ProjectName,
        description: Option<ProjectDescription>,
        organization_id: OrganizationId,
    ) -> DomainResult<Self> {
        let now = clock::now();
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
        self.updated_at = clock::now();
        Ok(())
    }

    pub fn update_description(
        &mut self,
        description: Option<ProjectDescription>,
    ) -> DomainResult<()> {
        self.description = description;
        self.updated_at = clock::now();
        Ok(())
    }

    /// Set the active flag to an explicit value. Idempotent on purpose: setting
    /// it to what it already is succeeds and is a no-op, so a retried or
    /// double-submitted request lands on the state the caller asked for. Unlike
    /// [`Self::activate`] / [`Self::deactivate`], it never errors on a no-op.
    pub fn set_active(&mut self, is_active: bool) {
        if self.is_active == is_active {
            return;
        }
        self.is_active = is_active;
        self.updated_at = clock::now();
    }

    pub fn deactivate(&mut self) -> DomainResult<()> {
        if !self.is_active {
            return Err(DomainError::business_rule("Project is already inactive"));
        }
        self.is_active = false;
        self.updated_at = clock::now();
        Ok(())
    }

    pub fn activate(&mut self) -> DomainResult<()> {
        if self.is_active {
            return Err(DomainError::business_rule("Project is already active"));
        }
        self.is_active = true;
        self.updated_at = clock::now();
        Ok(())
    }

    #[must_use]
    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &ProjectName {
        &self.name
    }

    #[must_use]
    pub fn description(&self) -> Option<&ProjectDescription> {
        self.description.as_ref()
    }

    #[must_use]
    pub fn organization_id(&self) -> &OrganizationId {
        &self.organization_id
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
