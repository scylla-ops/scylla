use crate::domain::entities::UserId;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::user::{Email, PasswordHash, Username};
use crate::domain::clock;
use chrono::{DateTime, Utc};

/// User domain entity
#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    username: Username,
    /// Optional so legacy/username-only accounts remain valid; required at
    /// signup and used for email login, mail and OAuth linking.
    email: Option<Email>,
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
        email: Option<Email>,
        password_hash: PasswordHash,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            is_active,
            created_at,
            updated_at,
        }
    }

    #[must_use]
    pub fn create(username: Username, email: Option<Email>, password_hash: PasswordHash) -> Self {
        let now = clock::now();
        Self {
            id: UserId::generate(),
            username,
            email,
            password_hash,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_username(&mut self, username: Username) -> DomainResult<()> {
        self.username = username;
        self.updated_at = clock::now();
        Ok(())
    }

    pub fn set_email(&mut self, email: Option<Email>) {
        self.email = email;
        self.updated_at = clock::now();
    }

    pub fn update_password_hash(&mut self, password_hash: PasswordHash) {
        self.password_hash = password_hash;
        self.updated_at = clock::now();
    }

    pub fn deactivate(&mut self) -> DomainResult<()> {
        if !self.is_active {
            return Err(DomainError::business_rule("User is already inactive"));
        }
        self.is_active = false;
        self.updated_at = clock::now();
        Ok(())
    }

    pub fn activate(&mut self) -> DomainResult<()> {
        if self.is_active {
            return Err(DomainError::business_rule("User is already active"));
        }
        self.is_active = true;
        self.updated_at = clock::now();
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
    pub fn email(&self) -> Option<&Email> {
        self.email.as_ref()
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
