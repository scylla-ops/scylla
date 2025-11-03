use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{UserId, Username};
use chrono::{DateTime, Utc};
use derive_more::Constructor;

/// User domain entity
#[derive(Debug, Clone, Constructor)]
pub struct User {
    id: UserId,
    username: Username,
    password_hash: String,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user
    pub fn create(username: Username, password_hash: String) -> Self {
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

    /// Update the username
    pub fn update_username(&mut self, username: Username) -> DomainResult<()> {
        self.username = username;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update the password hash
    pub fn update_password_hash(&mut self, password_hash: String) -> DomainResult<()> {
        if password_hash.is_empty() {
            return Err(DomainError::validation("Password hash cannot be empty"));
        }
        self.password_hash = password_hash;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Deactivate the user
    pub fn deactivate(&mut self) -> DomainResult<()> {
        if !self.is_active {
            return Err(DomainError::business_rule("User is already inactive"));
        }
        self.is_active = false;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Activate the user
    pub fn activate(&mut self) -> DomainResult<()> {
        if self.is_active {
            return Err(DomainError::business_rule("User is already active"));
        }
        self.is_active = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    // Getters
    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn username(&self) -> &Username {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test username
    fn test_username() -> Username {
        Username::new("testuser".to_string()).unwrap()
    }

    /// Helper to create a different test username
    fn other_username() -> Username {
        Username::new("otheruser".to_string()).unwrap()
    }

    // ===== Tests for User::create() =====

    #[test]
    fn test_create_user_sets_default_values() {
        let username = test_username();
        let password_hash = "hashed_password".to_string();

        let user = User::create(username.clone(), password_hash.clone());

        // Verify the user is created with expected defaults
        assert_eq!(user.username().as_str(), "testuser");
        assert_eq!(user.password_hash(), "hashed_password");
        assert!(user.is_active(), "New user should be active by default");

        // Verify timestamps are set (created_at and updated_at should be equal for new user)
        assert_eq!(user.created_at(), user.updated_at());
    }

    #[test]
    fn test_create_user_generates_unique_id() {
        let username1 = test_username();
        let username2 = test_username();

        let user1 = User::create(username1, "hash1".to_string());
        let user2 = User::create(username2, "hash2".to_string());

        // IDs should be different even with same username (uniqueness is repository's concern)
        assert_ne!(user1.id().as_str(), user2.id().as_str());
    }

    // ===== Tests for User::activate() =====

    #[test]
    fn test_activate_inactive_user_succeeds() {
        let mut user = User::create(test_username(), "hash".to_string());

        // First deactivate
        user.deactivate().unwrap();
        assert!(!user.is_active());

        let original_updated_at = user.updated_at();

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(1));

        // Now activate
        let result = user.activate();

        assert!(result.is_ok(), "Activating inactive user should succeed");
        assert!(user.is_active(), "User should be active after activation");
        assert!(
            user.updated_at() > original_updated_at,
            "updated_at should be updated when user is activated"
        );
    }

    #[test]
    fn test_activate_already_active_user_fails() {
        let mut user = User::create(test_username(), "hash".to_string());

        // User is already active by default
        assert!(user.is_active());

        let result = user.activate();

        // Should return business rule error
        assert!(
            result.is_err(),
            "Activating already active user should fail"
        );

        if let Err(DomainError::BusinessRule(msg)) = result {
            assert_eq!(msg, "User is already active");
        } else {
            panic!("Expected BusinessRule error");
        }
    }

    // ===== Tests for User::deactivate() =====

    #[test]
    fn test_deactivate_active_user_succeeds() {
        let mut user = User::create(test_username(), "hash".to_string());

        assert!(user.is_active());
        let original_updated_at = user.updated_at();

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(1));

        let result = user.deactivate();

        assert!(result.is_ok(), "Deactivating active user should succeed");
        assert!(
            !user.is_active(),
            "User should be inactive after deactivation"
        );
        assert!(
            user.updated_at() > original_updated_at,
            "updated_at should be updated when user is deactivated"
        );
    }

    #[test]
    fn test_deactivate_already_inactive_user_fails() {
        let mut user = User::create(test_username(), "hash".to_string());

        // First deactivate
        user.deactivate().unwrap();
        assert!(!user.is_active());

        // Try to deactivate again
        let result = user.deactivate();

        assert!(
            result.is_err(),
            "Deactivating already inactive user should fail"
        );

        if let Err(DomainError::BusinessRule(msg)) = result {
            assert_eq!(msg, "User is already inactive");
        } else {
            panic!("Expected BusinessRule error");
        }
    }

    // ===== Tests for User::update_username() =====

    #[test]
    fn test_update_username_succeeds() {
        let mut user = User::create(test_username(), "hash".to_string());

        let original_updated_at = user.updated_at();

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(1));

        let new_username = other_username();
        let result = user.update_username(new_username.clone());

        assert!(result.is_ok(), "Updating username should succeed");
        assert_eq!(user.username().as_str(), "otheruser");
        assert!(
            user.updated_at() > original_updated_at,
            "updated_at should be updated when username changes"
        );
    }

    // ===== Tests for User::update_password_hash() =====

    #[test]
    fn test_update_password_hash_with_valid_hash_succeeds() {
        let mut user = User::create(test_username(), "old_hash".to_string());

        let original_updated_at = user.updated_at();

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(1));

        let result = user.update_password_hash("new_hash".to_string());

        assert!(result.is_ok(), "Updating password hash should succeed");
        assert_eq!(user.password_hash(), "new_hash");
        assert!(
            user.updated_at() > original_updated_at,
            "updated_at should be updated when password hash changes"
        );
    }

    #[test]
    fn test_update_password_hash_with_empty_hash_fails() {
        let mut user = User::create(test_username(), "hash".to_string());

        let result = user.update_password_hash("".to_string());

        assert!(
            result.is_err(),
            "Updating with empty password hash should fail"
        );

        if let Err(DomainError::Validation(msg)) = result {
            assert_eq!(msg, "Password hash cannot be empty");
        } else {
            panic!("Expected Validation error");
        }

        // Original hash should be unchanged
        assert_eq!(user.password_hash(), "hash");
    }

    // ===== Tests for User::new() (reconstruction from database) =====

    #[test]
    fn test_new_reconstructs_user_from_database() {
        let id = UserId::generate();
        let username = test_username();
        let password_hash = "stored_hash".to_string();
        let is_active = false;
        let created_at = Utc::now();
        let updated_at = Utc::now();

        let result = User::new(
            id.clone(),
            username,
            password_hash,
            is_active,
            created_at,
            updated_at,
        );

        assert!(
            result.is_ok(),
            "Reconstructing user from database should succeed"
        );

        let user = result.unwrap();
        assert_eq!(user.id().as_str(), id.as_str());
        assert!(!user.is_active());
        assert_eq!(user.created_at(), created_at);
        assert_eq!(user.updated_at(), updated_at);
    }

    // ===== Integration tests for state transitions =====

    #[test]
    fn test_user_lifecycle_activate_deactivate_cycle() {
        let mut user = User::create(test_username(), "hash".to_string());

        // User starts active
        assert!(user.is_active());

        // Deactivate
        user.deactivate()
            .expect("First deactivation should succeed");
        assert!(!user.is_active());

        // Can't deactivate again
        assert!(user.deactivate().is_err());

        // Activate
        user.activate().expect("Activation should succeed");
        assert!(user.is_active());

        // Can't activate again
        assert!(user.activate().is_err());
    }
}
