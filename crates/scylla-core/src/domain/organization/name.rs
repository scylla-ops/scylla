use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

const MAX_NAME_LENGTH: usize = 255;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("Organization name cannot be empty"));
    }
    if s.len() > MAX_NAME_LENGTH {
        return Err(DomainError::validation(format!(
            "Organization name cannot exceed {MAX_NAME_LENGTH} characters"
        )));
    }
    Ok(())
}

#[nutype(
    sanitize(trim),
    validate(with = validate, error = DomainError),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Borrow, Display, Into),
)]
pub struct OrganizationName(String);

impl OrganizationName {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        Self::try_new(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        <Self as AsRef<str>>::as_ref(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_and_trimming() {
        assert!(OrganizationName::new("Valid Org").is_ok());
        let trimmed = OrganizationName::new("  My Org  ").unwrap();
        assert_eq!(trimmed.as_str(), "My Org");
        assert!(OrganizationName::new("").is_err());
        assert!(OrganizationName::new("   ").is_err());
    }

    #[test]
    fn enforces_length_bounds() {
        let max = OrganizationName::new("a".repeat(MAX_NAME_LENGTH)).unwrap();
        assert_eq!(max.as_str().len(), MAX_NAME_LENGTH);
        assert!(OrganizationName::new("a".repeat(MAX_NAME_LENGTH + 1)).is_err());
    }
}
