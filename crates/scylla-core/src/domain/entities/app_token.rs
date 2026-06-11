use crate::domain::clock;
use crate::domain::entities::{AppCredentialId, AppId, AppTokenId};
use chrono::{DateTime, Duration, Utc};

/// A bearer token issued to a machine [`App`](super::App) after it presents the
/// plaintext of one of its secrets. Kept separate from user `Session`s: the
/// auth interceptor resolves a token to an `App` principal. Carries the
/// `secret_id` that minted it, so revoking that secret cascades the token away
/// and disabling it lets the lookup reject the token at once. Cascades when its
/// app (or that secret) is deleted.
#[derive(Debug, Clone)]
pub struct AppToken {
    id: AppTokenId,
    token: String,
    app_id: AppId,
    secret_id: AppCredentialId,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl AppToken {
    #[must_use]
    pub fn from_persistence(
        id: AppTokenId,
        token: String,
        app_id: AppId,
        secret_id: AppCredentialId,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            token,
            app_id,
            secret_id,
            created_at,
            expires_at,
        }
    }

    #[must_use]
    pub fn create(
        app_id: AppId,
        secret_id: AppCredentialId,
        token: String,
        duration: Duration,
    ) -> Self {
        let now = clock::now();
        Self {
            id: AppTokenId::generate(),
            token,
            app_id,
            secret_id,
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
    pub fn secret_id(&self) -> &AppCredentialId {
        &self.secret_id
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
