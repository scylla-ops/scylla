use crate::domain::clock;
use crate::domain::entities::{AppId, AppTokenId};
use chrono::{DateTime, Duration, Utc};

/// A bearer token issued to a machine [`App`](super::App) after it presents its
/// secret. Kept separate from user `Session`s: the auth interceptor resolves a
/// token to an `App` principal. Cascades when its app is deleted.
#[derive(Debug, Clone)]
pub struct AppToken {
    id: AppTokenId,
    token: String,
    app_id: AppId,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl AppToken {
    #[must_use]
    pub fn from_persistence(
        id: AppTokenId,
        token: String,
        app_id: AppId,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            token,
            app_id,
            created_at,
            expires_at,
        }
    }

    #[must_use]
    pub fn create(app_id: AppId, token: String, duration: Duration) -> Self {
        let now = clock::now();
        Self {
            id: AppTokenId::generate(),
            token,
            app_id,
            created_at: now,
            expires_at: now + duration,
        }
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        clock::now() > self.expires_at
    }

    #[must_use]
    pub fn id(&self) -> &AppTokenId {
        &self.id
    }

    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[must_use]
    pub fn app_id(&self) -> &AppId {
        &self.app_id
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
}
