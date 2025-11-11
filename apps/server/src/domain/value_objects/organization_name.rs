use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

const MAX_NAME_LENGTH: usize = 255;

/// OrganizationName value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrganizationName {
    inner: String,
}

impl OrganizationName {
    /// Create a new OrganizationName with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Organization name cannot be empty"));
        }

        if trimmed.len() > MAX_NAME_LENGTH {
            return Err(DomainError::validation(format!(
                "Organization name cannot exceed {} characters",
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

impl fmt::Display for OrganizationName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for OrganizationName {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl PartialEq<str> for OrganizationName {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<&str> for OrganizationName {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_name_creation() {
        assert!(OrganizationName::new("Valid Org").is_ok());
        assert!(OrganizationName::new("  Valid Org  ").is_ok()); // trimming
        assert!(OrganizationName::new("").is_err()); // empty
        assert!(OrganizationName::new("   ").is_err()); // whitespace only
    }

    #[test]
    fn test_organization_name_validation() {
        // Valid names
        assert!(OrganizationName::new("My Organization").is_ok());
        assert!(OrganizationName::new("A").is_ok());

        // Invalid names
        assert!(OrganizationName::new("").is_err());
        assert!(OrganizationName::new("   ").is_err());

        // Too long
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        assert!(OrganizationName::new(long_name).is_err());

        // Exactly max length should be ok
        let max_name = "a".repeat(MAX_NAME_LENGTH);
        assert!(OrganizationName::new(max_name).is_ok());
    }

    #[test]
    fn test_organization_name_trimming() {
        let name = OrganizationName::new("  My Org  ").unwrap();
        assert_eq!(name.as_str(), "My Org");
    }

    #[test]
    fn test_organization_name_comparison() {
        let name = OrganizationName::new("My Org").unwrap();
        assert_eq!(name, "My Org");
        assert_eq!(name.as_str(), "My Org");
    }
}
