use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

const MAX_LABEL_LENGTH: usize = 64;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("App secret label cannot be empty"));
    }
    if s.len() > MAX_LABEL_LENGTH {
        return Err(DomainError::validation(format!(
            "App secret label cannot exceed {MAX_LABEL_LENGTH} characters"
        )));
    }
    Ok(())
}

/// Human-readable label distinguishing one secret of an App from another
/// (e.g. `default`, `ci-runner`). Unique within its owning App.
#[nutype(
    sanitize(trim),
    validate(with = validate, error = DomainError),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Borrow, Display, Into),
)]
pub struct AppSecretLabel(String);

impl AppSecretLabel {
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
        assert!(AppSecretLabel::new("ci-runner").is_ok());
        assert_eq!(
            AppSecretLabel::new("  default  ").unwrap().as_str(),
            "default"
        );
        assert!(AppSecretLabel::new("").is_err());
        assert!(AppSecretLabel::new("   ").is_err());
    }

    #[test]
    fn enforces_length_bound() {
        assert!(AppSecretLabel::new("a".repeat(MAX_LABEL_LENGTH)).is_ok());
        assert!(AppSecretLabel::new("a".repeat(MAX_LABEL_LENGTH + 1)).is_err());
    }
}
