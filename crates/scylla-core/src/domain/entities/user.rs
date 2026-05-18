use crate::domain::entities::UserId;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::user::{PasswordHash, Username};
use chrono::{DateTime, Utc};

/// User domain entity
#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    username: Username,
    password_hash: PasswordHash,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    /// Reconstitute a `User` from persistent storage. Skips creation-time
    /// invariants — the caller is the trusted repository layer.
    #[must_use]
    pub fn from_persistence(
        id: UserId,
        username: Username,
        password_hash: PasswordHash,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            password_hash,
            is_active,
            created_at,
            updated_at,
        }
    }

    #[must_use]
    pub fn create(username: Username, password_hash: PasswordHash) -> Self {
        let now = Utc::now();
        Self {
            id: UserId::generate(),
            username,
            password_hash,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_username(&mut self, username: Username) -> DomainResult<()> {
        self.username = username;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_password_hash(&mut self, password_hash: PasswordHash) {
        self.password_hash = password_hash;
        self.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) -> DomainResult<()> {
        if !self.is_active {
            return Err(DomainError::business_rule("User is already inactive"));
        }
        self.is_active = false;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn activate(&mut self) -> DomainResult<()> {
        if self.is_active {
            return Err(DomainError::business_rule("User is already active"));
        }
        self.is_active = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    #[must_use]
    pub fn id(&self) -> &UserId {
        &self.id
    }

    #[must_use]
    pub fn username(&self) -> &Username {
        &self.username
    }

    #[must_use]
    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
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
