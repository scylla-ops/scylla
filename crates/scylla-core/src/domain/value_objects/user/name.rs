use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

const MAX_USERNAME_LENGTH: usize = 255;

/// Username value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct Username {
    inner: String,
}

impl Username {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Username cannot be empty"));
        }

        if trimmed.len() > MAX_USERNAME_LENGTH {
            return Err(DomainError::validation(format!(
                "Username cannot exceed {MAX_USERNAME_LENGTH} characters"
            )));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
    }
}

impl Username {
    #[must_use] 
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    #[must_use] 
    pub fn into_string(self) -> String {
        self.inner
    }

    #[must_use] 
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[must_use] 
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
        assert!(Username::new("  valid_user  ").is_ok());
        assert!(Username::new("").is_err());
        assert!(Username::new("   ").is_err());
    }

    #[test]
    fn test_username_validation() {
        assert!(Username::new("user123").is_ok());
        assert!(Username::new("A").is_ok());
        assert!(Username::new("").is_err());
        assert!(Username::new("   ").is_err());

        let long_username = "a".repeat(MAX_USERNAME_LENGTH + 1);
        assert!(Username::new(long_username).is_err());

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
