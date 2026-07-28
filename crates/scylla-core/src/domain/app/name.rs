use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

const MAX_NAME_LENGTH: usize = 255;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("App name cannot be empty"));
    }
    if s.len() > MAX_NAME_LENGTH {
        return Err(DomainError::validation(format!(
            "App name cannot exceed {MAX_NAME_LENGTH} characters"
        )));
    }
    Ok(())
}

/// Human-readable label for a machine App (agent / automation), unique within
/// its owning organization.
#[nutype(
    sanitize(trim),
    validate(with = validate, error = DomainError),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Borrow, Display, Into),
)]
pub struct AppName(String);

impl AppName {
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
        assert!(AppName::new("ci-runner").is_ok());
        assert_eq!(AppName::new("  bot  ").unwrap().as_str(), "bot");
        assert!(AppName::new("").is_err());
        assert!(AppName::new("   ").is_err());
    }

    #[test]
    fn enforces_length_bound() {
        assert!(AppName::new("a".repeat(MAX_NAME_LENGTH)).is_ok());
        assert!(AppName::new("a".repeat(MAX_NAME_LENGTH + 1)).is_err());
    }
}
