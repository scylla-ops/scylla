use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

const MAX_USERNAME_LENGTH: usize = 255;

/// Username value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username {
    inner: String,
}

impl Username {
    /// Create a new Username with validation
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

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl PartialEq<str> for Username {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<&str> for Username {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_creation() {
        assert!(Username::new("valid_user").is_ok());
        assert!(Username::new("  valid_user  ").is_ok()); // trimming
        assert!(Username::new("").is_err()); // empty
        assert!(Username::new("   ").is_err()); // whitespace only
    }

    #[test]
    fn test_username_validation() {
        // Valid usernames
        assert!(Username::new("user123").is_ok());
        assert!(Username::new("A").is_ok());

        // Invalid usernames
        assert!(Username::new("").is_err());
        assert!(Username::new("   ").is_err());

        // Too long
        let long_username = "a".repeat(MAX_USERNAME_LENGTH + 1);
        assert!(Username::new(long_username).is_err());

        // Exactly max length should be ok
        let max_username = "a".repeat(MAX_USERNAME_LENGTH);
        assert!(Username::new(max_username).is_ok());
    }

    #[test]
    fn test_username_trimming() {
        let username = Username::new("  myuser  ").unwrap();
        assert_eq!(username.as_str(), "myuser");
    }

    #[test]
    fn test_username_comparison() {
        let username = Username::new("myuser").unwrap();
        assert_eq!(username, "myuser");
        assert_eq!(username.as_str(), "myuser");
    }
}
