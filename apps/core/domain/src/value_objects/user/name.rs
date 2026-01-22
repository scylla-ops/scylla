use crate::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::fmt;

const MAX_USERNAME_LENGTH: usize = 255;

/// Username value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserName {
    inner: String,
}

impl UserName {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Username cannot be empty"));
        }

        if trimmed.len() > MAX_USERNAME_LENGTH {
            return Err(DomainError::validation(format!(
                "Username cannot exceed {} characters",
                MAX_USERNAME_LENGTH
            )));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
    }
}

impl TryFrom<String> for UserName {
    type Error = DomainError;

    fn try_from(value: String) -> DomainResult<Self> {
        UserName::new(value)
    }
}

impl UserName {
    /// Get the username as a string slice
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to inner String
    pub fn into_string(self) -> String {
        self.inner
    }

    /// Get the length of the username
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the username is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl fmt::Display for UserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for UserName {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl PartialEq<str> for UserName {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<&str> for UserName {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_creation() {
        assert!(UserName::try_from("valid_user".to_string()).is_ok());
        assert!(UserName::try_from("  valid_user  ".to_string()).is_ok()); // trimming
        assert!(UserName::try_from("".to_string()).is_err()); // empty
        assert!(UserName::try_from("   ".to_string()).is_err()); // whitespace only
    }

    #[test]
    fn test_username_validation() {
        // Valid usernames
        assert!(UserName::try_from("user123".to_string()).is_ok());
        assert!(UserName::try_from("A".to_string()).is_ok());

        // Invalid usernames
        assert!(UserName::try_from("".to_string()).is_err());
        assert!(UserName::try_from("   ".to_string()).is_err());

        // Too long
        let long_username = "a".repeat(MAX_USERNAME_LENGTH + 1);
        assert!(UserName::try_from(long_username).is_err());

        // Exactly max length should be ok
        let max_username = "a".repeat(MAX_USERNAME_LENGTH);
        assert!(UserName::try_from(max_username).is_ok());
    }

    #[test]
    fn test_username_trimming() {
        let username = UserName::try_from("  myuser  ".to_string()).unwrap();
        assert_eq!(username.as_str(), "myuser");
    }

    #[test]
    fn test_username_comparison() {
        let username = UserName::try_from("myuser".to_string()).unwrap();
        assert_eq!(username, "myuser");
        assert_eq!(username.as_str(), "myuser");
    }
}
