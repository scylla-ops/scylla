use crate::domain::clock;
use crate::domain::entities::ids::{AppId, OrganizationId};
use crate::domain::value_objects::app::{AppName, AppSecretHash};
use chrono::{DateTime, Utc};

/// A machine principal owned by an organization (an agent or automation). It
/// holds its credential only as a hash — the plaintext [`AppSecret`] is shown
/// once at creation and never persisted. An App authenticates with an app token
/// and acts under scoped grants (typically the `worker` role on its org).
///
/// [`AppSecret`]: crate::domain::value_objects::app::AppSecret
#[derive(Debug, Clone)]
pub struct App {
    id: AppId,
    organization_id: OrganizationId,
    name: AppName,
    secret_hash: AppSecretHash,
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
        secret_hash: AppSecretHash,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            organization_id,
            name,
            secret_hash,
            is_active,
            created_at,
            updated_at,
        }
    }

    #[must_use]
    pub fn create(
        organization_id: OrganizationId,
        name: AppName,
        secret_hash: AppSecretHash,
    ) -> Self {
        let now = clock::now();
        Self {
            id: AppId::generate(),
            organization_id,
            name,
            secret_hash,
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
    pub fn secret_hash(&self) -> &AppSecretHash {
        &self.secret_hash
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
