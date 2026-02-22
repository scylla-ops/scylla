use crate::errors::{DomainError, DomainResult};
use std::fmt;

const MAX_NAME_LENGTH: usize = 255;

/// OrganizationName value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrganizationName {
    inner: String,
}

impl OrganizationName {
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

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn into_string(self) -> String {
        self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

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

#[cfg(feature = "surrealdb")]
impl surrealdb_types::SurrealValue for OrganizationName {
    fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::String
    }

    fn into_value(self) -> surrealdb_types::Value {
        surrealdb_types::Value::String(self.inner)
    }

    fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb_types::Error> {
        match value {
            surrealdb_types::Value::String(s) => Self::new(s).map_err(|e| {
                surrealdb_types::Error::internal(format!("Invalid OrganizationName: {}", e))
            }),
            other => {
                Err(surrealdb_types::ConversionError::from_value(Self::kind_of(), &other).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_name_creation() {
        assert!(OrganizationName::new("Valid Org").is_ok());
        assert!(OrganizationName::new("  Valid Org  ").is_ok());
        assert!(OrganizationName::new("").is_err());
        assert!(OrganizationName::new("   ").is_err());
    }

    #[test]
    fn test_organization_name_validation() {
        assert!(OrganizationName::new("My Organization").is_ok());
        assert!(OrganizationName::new("A").is_ok());
        assert!(OrganizationName::new("").is_err());
        assert!(OrganizationName::new("   ").is_err());

        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        assert!(OrganizationName::new(long_name).is_err());

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
