use crate::entities::{SessionId, UserId};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Session domain entity for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    id: SessionId,
    token: String,
    user_id: UserId,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_active_at: DateTime<Utc>,
}

impl Session {
    /// Create a new session with a random token
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

    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Update the last active timestamp
    pub fn touch(&mut self) {
        self.last_active_at = Utc::now();
    }

    /// Extend the session expiration
    pub fn extend(&mut self, duration: Duration) {
        self.expires_at = Utc::now() + duration;
        self.last_active_at = Utc::now();
    }

    // Getters
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn last_active_at(&self) -> DateTime<Utc> {
        self.last_active_at
    }
}
