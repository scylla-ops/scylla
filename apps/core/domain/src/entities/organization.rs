use crate::entities::ids::OrganizationId;
use crate::errors::{DomainError, DomainResult};
use crate::value_objects::organization::{OrganizationDescription, OrganizationName};
use chrono::{DateTime, Utc};
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

/// Organization domain entity
#[derive(Debug, Clone)]
#[cfg_attr(feature = "surrealdb", derive(surrealdb_types::SurrealValue))]
pub struct Organization {
    id: OrganizationId,
    name: OrganizationName,
    description: Option<OrganizationDescription>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Organization {
    pub fn create(
        name: OrganizationName,
        description: Option<OrganizationDescription>,
    ) -> DomainResult<Self> {
        let now = Utc::now();
        Ok(Self {
            id: OrganizationId::generate(),
            name,
            description,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_name(&mut self, name: OrganizationName) -> DomainResult<()> {
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_description(
        &mut self,
        description: Option<OrganizationDescription>,
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
            return Err(DomainError::business_rule(
                "Organization is already inactive",
            ));
        }
        self.is_active = false;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn activate(&mut self) -> DomainResult<()> {
        if self.is_active {
            return Err(DomainError::business_rule("Organization is already active"));
        }
        self.is_active = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn id(&self) -> &OrganizationId {
        &self.id
    }

    pub fn name(&self) -> &OrganizationName {
        &self.name
    }

    pub fn description(&self) -> Option<&OrganizationDescription> {
        self.description.as_ref()
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
