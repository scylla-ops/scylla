use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Description value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Description {
    inner: String,
}

impl Description {
    /// Create a new Description with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.len() > MAX_DESCRIPTION_LENGTH {
            return Err(DomainError::validation(format!(
                "Description cannot exceed {} characters",
                MAX_DESCRIPTION_LENGTH
            )));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
    }

    /// Create an empty description (for None case)
    pub fn empty() -> Self {
        Self {
            inner: String::new(),
        }
    }

    /// Check if the description is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the description as a string slice
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to inner String
    pub fn into_string(self) -> String {
        self.inner
    }

    /// Get the length of the description
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl fmt::Display for Description {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for Description {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl PartialEq<str> for Description {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<&str> for Description {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

impl From<Option<String>> for Description {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(desc) => Self::new(desc).unwrap_or_else(|_| Self::empty()),
            None => Self::empty(),
        }
    }
}

impl From<Description> for Option<String> {
    fn from(value: Description) -> Self {
        if value.is_empty() {
            None
        } else {
            Some(value.into_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_description_creation() {
        assert!(Description::new("Valid description").is_ok());
        assert!(Description::new("  Valid description  ").is_ok()); // trimming
        assert!(Description::new("").is_ok()); // empty allowed
        assert!(Description::new("   ").is_ok()); // whitespace only allowed
    }

    #[test]
    fn test_description_validation() {
        // Valid descriptions
        assert!(Description::new("My Description").is_ok());
        assert!(Description::new("").is_ok());

        // Too long
        let long_description = "a".repeat(MAX_DESCRIPTION_LENGTH + 1);
        assert!(Description::new(long_description).is_err());

        // Exactly max length should be ok
        let max_description = "a".repeat(MAX_DESCRIPTION_LENGTH);
        assert!(Description::new(max_description).is_ok());
    }

    #[test]
    fn test_description_trimming() {
        let desc = Description::new("  My Description  ").unwrap();
        assert_eq!(desc.as_str(), "My Description");
    }

    #[test]
    fn test_description_empty() {
        let empty = Description::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_description_from_option() {
        let desc = Description::from(Some("test".to_string()));
        assert_eq!(desc.as_str(), "test");

        let empty = Description::from(None);
        assert!(empty.is_empty());

        let option: Option<String> = Description::from(Some("test".to_string())).into();
        assert_eq!(option, Some("test".to_string()));

        let none: Option<String> = Description::empty().into();
        assert_eq!(none, None);
    }
}
