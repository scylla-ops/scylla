use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

const MAX_NAME_LENGTH: usize = 255;

/// ProjectName value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectName {
    inner: String,
}

impl ProjectName {
    /// Create a new ProjectName with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Project name cannot be empty"));
        }

        if trimmed.len() > MAX_NAME_LENGTH {
            return Err(DomainError::validation(format!(
                "Project name cannot exceed {} characters",
                MAX_NAME_LENGTH
            )));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
    }

    /// Get the name as a string slice
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to inner String
    pub fn into_string(self) -> String {
        self.inner
    }

    /// Get the length of the name
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the name is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl fmt::Display for ProjectName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for ProjectName {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl PartialEq<str> for ProjectName {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<&str> for ProjectName {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_name_creation() {
        assert!(ProjectName::new("Valid Project").is_ok());
        assert!(ProjectName::new("  Valid Project  ").is_ok()); // trimming
        assert!(ProjectName::new("").is_err()); // empty
        assert!(ProjectName::new("   ").is_err()); // whitespace only
    }

    #[test]
    fn test_project_name_validation() {
        // Valid names
        assert!(ProjectName::new("My Project").is_ok());
        assert!(ProjectName::new("A").is_ok());

        // Invalid names
        assert!(ProjectName::new("").is_err());
        assert!(ProjectName::new("   ").is_err());

        // Too long
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        assert!(ProjectName::new(long_name).is_err());

        // Exactly max length should be ok
        let max_name = "a".repeat(MAX_NAME_LENGTH);
        assert!(ProjectName::new(max_name).is_ok());
    }

    #[test]
    fn test_project_name_trimming() {
        let name = ProjectName::new("  My Project  ").unwrap();
        assert_eq!(name.as_str(), "My Project");
    }

    #[test]
    fn test_project_name_comparison() {
        let name = ProjectName::new("My Project").unwrap();
        assert_eq!(name, "My Project");
        assert_eq!(name.as_str(), "My Project");
    }
}
