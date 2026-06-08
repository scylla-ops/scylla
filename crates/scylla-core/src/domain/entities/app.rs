use crate::domain::clock;
use crate::domain::entities::ids::{AppId, OrganizationId};
use crate::domain::value_objects::app::AppName;
use chrono::{DateTime, Utc};

/// A machine principal owned by an organization (an agent or automation). Its
/// credentials live separately as one or more [`AppCredential`]s — an App is
/// just an identity here. It authenticates with an app token and acts under
/// scoped grants (typically the `agent` role on its org).
///
/// [`AppCredential`]: super::AppCredential
#[derive(Debug, Clone)]
pub struct App {
    id: AppId,
    organization_id: OrganizationId,
    name: AppName,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl App {
    #[must_use]
    pub fn from_persistence(
        id: AppId,
        organization_id: OrganizationId,
        name: AppName,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            organization_id,
            name,
            is_active,
            created_at,
            updated_at,
        }
    }

    #[must_use]
    pub fn create(organization_id: OrganizationId, name: AppName) -> Self {
        let now = clock::now();
        Self {
            id: AppId::generate(),
            organization_id,
            name,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[must_use]
    pub fn id(&self) -> &AppId {
        &self.id
    }

    #[must_use]
    pub fn organization_id(&self) -> &OrganizationId {
        &self.organization_id
    }

    #[must_use]
    pub fn name(&self) -> &AppName {
        &self.name
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
