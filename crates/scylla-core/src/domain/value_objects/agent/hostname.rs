use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

const MAX_HOSTNAME_LENGTH: usize = 255;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("Hostname cannot be empty"));
    }
    if s.len() > MAX_HOSTNAME_LENGTH {
        return Err(DomainError::validation(format!(
            "Hostname cannot exceed {MAX_HOSTNAME_LENGTH} characters"
        )));
    }
    Ok(())
}

#[nutype(
    sanitize(trim),
    validate(with = validate, error = DomainError),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Borrow, Display, Into),
)]
pub struct Hostname(String);

impl Hostname {
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
    fn rejects_empty_and_whitespace() {
        assert!(matches!(Hostname::new(""), Err(DomainError::Validation(_))));
        assert!(matches!(
            Hostname::new("   \t\n"),
            Err(DomainError::Validation(_))
        ));
    }

    #[test]
    fn trims_whitespace() {
        let h = Hostname::new("  host-1  ").unwrap();
        assert_eq!(h.as_str(), "host-1");
    }

    #[test]
    fn enforces_max_length() {
        assert!(Hostname::new("a".repeat(MAX_HOSTNAME_LENGTH)).is_ok());
        assert!(matches!(
            Hostname::new("a".repeat(MAX_HOSTNAME_LENGTH + 1)),
            Err(DomainError::Validation(_))
        ));
    }

    #[test]
    fn display_matches_inner() {
        let h = Hostname::new("box").unwrap();
        assert_eq!(format!("{h}"), "box");
    }
}
