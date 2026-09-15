use crate::domain::app::{AppSecretHash, AppSecretLabel};
use crate::domain::clock;
use crate::domain::ids::{AppCredentialId, AppId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct AppCredential {
    id: AppCredentialId,
    app_id: AppId,
    label: AppSecretLabel,
    secret_hash: AppSecretHash,
    enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl AppCredential {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn from_persistence(
        id: AppCredentialId,
        app_id: AppId,
        label: AppSecretLabel,
        secret_hash: AppSecretHash,
        enabled: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            app_id,
            label,
            secret_hash,
            enabled,
            created_at,
            updated_at,
        }
    }

    #[must_use]
    pub fn create(app_id: AppId, label: AppSecretLabel, secret_hash: AppSecretHash) -> Self {
        let now = clock::now();
        Self {
            id: AppCredentialId::generate(),
            app_id,
            label,
            secret_hash,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[must_use]
    pub fn id(&self) -> &AppCredentialId {
        &self.id
    }

    #[must_use]
    pub fn app_id(&self) -> &AppId {
        &self.app_id
    }

    #[must_use]
    pub fn label(&self) -> &AppSecretLabel {
        &self.label
    }

    #[must_use]
    pub fn secret_hash(&self) -> &AppSecretHash {
        &self.secret_hash
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
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
