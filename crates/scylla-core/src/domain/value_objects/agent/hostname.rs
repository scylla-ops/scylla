use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

const MAX_HOSTNAME_LENGTH: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct Hostname {
    inner: String,
}

impl Hostname {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Hostname cannot be empty"));
        }

        if trimmed.len() > MAX_HOSTNAME_LENGTH {
            return Err(DomainError::validation(format!(
                "Hostname cannot exceed {MAX_HOSTNAME_LENGTH} characters"
            )));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl fmt::Display for Hostname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for Hostname {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty() {
        assert!(matches!(Hostname::new(""), Err(DomainError::Validation(_))));
    }

    #[test]
    fn rejects_whitespace_only() {
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
    fn accepts_max_length() {
        let s = "a".repeat(MAX_HOSTNAME_LENGTH);
        let h = Hostname::new(&s).unwrap();
        assert_eq!(h.as_str().len(), MAX_HOSTNAME_LENGTH);
    }

    #[test]
    fn rejects_over_max_length() {
        let s = "a".repeat(MAX_HOSTNAME_LENGTH + 1);
        assert!(matches!(Hostname::new(s), Err(DomainError::Validation(_))));
    }

    #[test]
    fn display_matches_inner() {
        let h = Hostname::new("box").unwrap();
        assert_eq!(format!("{h}"), "box");
    }
}
