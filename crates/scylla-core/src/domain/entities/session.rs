use crate::domain::entities::{SessionId, UserId};
use chrono::{DateTime, Duration, Utc};

/// Session domain entity for authentication
#[derive(Debug, Clone)]
pub struct Session {
    id: SessionId,
    token: String,
    user_id: UserId,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_active_at: DateTime<Utc>,
}

impl Session {
    #[must_use]
    pub fn from_persistence(
        id: SessionId,
        token: String,
        user_id: UserId,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        last_active_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            token,
            user_id,
            created_at,
            expires_at,
            last_active_at,
        }
    }

    #[must_use]
    pub fn create(user_id: UserId, token: String, duration: Duration) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::generate(),
            token,
            user_id,
            created_at: now,
            expires_at: now + duration,
            last_active_at: now,
        }
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn touch(&mut self) {
        self.last_active_at = Utc::now();
    }

    pub fn extend(&mut self, duration: Duration) {
        self.expires_at = Utc::now() + duration;
        self.last_active_at = Utc::now();
    }

    #[must_use]
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    #[must_use]
    pub fn last_active_at(&self) -> DateTime<Utc> {
        self.last_active_at
    }
}
