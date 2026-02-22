use crate::entities::UserId;
use crate::errors::{DomainError, DomainResult};
use crate::value_objects::user::UserName;
use chrono::{DateTime, Utc};
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

/// User domain entity
#[derive(Debug, Clone)]
#[cfg_attr(feature = "surrealdb", derive(surrealdb_types::SurrealValue))]
pub struct User {
    id: UserId,
    username: UserName,
    password_hash: String,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn create(username: UserName, password_hash: String) -> Self {
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

    pub fn update_username(&mut self, username: UserName) -> DomainResult<()> {
        self.username = username;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_password_hash(&mut self, password_hash: String) -> DomainResult<()> {
        if password_hash.is_empty() {
            return Err(DomainError::validation("Password hash cannot be empty"));
        }
        self.password_hash = password_hash;
        self.updated_at = Utc::now();
        Ok(())
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

    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn username(&self) -> &UserName {
        &self.username
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
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
