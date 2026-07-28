use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;
use std::fmt;
use uuid::Uuid;

const SECRET_MIN_LENGTH: usize = 32;
const SECRET_MAX_LENGTH: usize = 512;

fn validate(s: &str) -> Result<(), DomainError> {
    let len = s.chars().count();
    if len < SECRET_MIN_LENGTH {
        return Err(DomainError::validation(format!(
            "App secret too short, minimum length is {SECRET_MIN_LENGTH}"
        )));
    }
    if len > SECRET_MAX_LENGTH {
        return Err(DomainError::validation(format!(
            "App secret too long, maximum length is {SECRET_MAX_LENGTH}"
        )));
    }
    Ok(())
}

/// Plaintext App credential, presented once at creation and again by the App
/// when it exchanges credentials for a token. Held only transiently; the stored
/// form is an [`AppSecretHash`](crate::domain::value_objects::app::AppSecretHash). `Debug` / `Display` are
/// masked so it never leaks into logs.
#[nutype(
    validate(with = validate, error = DomainError),
    derive(Clone, PartialEq, Eq, AsRef, Borrow, Into),
)]
pub struct AppSecret(String);

impl AppSecret {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        Self::try_new(value.into())
    }

    /// Generate a fresh high-entropy secret (256 bits, hex-encoded).
    #[must_use]
    pub fn generate() -> Self {
        let raw = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        // 64 hex chars always satisfies the length bounds.
        Self::try_new(raw).expect("generated secret is always valid")
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        <Self as AsRef<str>>::as_ref(self)
    }
}

impl fmt::Debug for AppSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppSecret")
            .field("inner", &"[REDACTED]")
            .finish()
    }
}

impl fmt::Display for AppSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_is_valid_and_unique() {
        let a = AppSecret::generate();
        let b = AppSecret::generate();
        assert_eq!(a.as_str().len(), 64);
        assert_ne!(a.as_str(), b.as_str());
    }

    #[test]
    fn rejects_too_short() {
        assert!(AppSecret::new("short").is_err());
    }

    #[test]
    fn debug_and_display_are_masked() {
        let s = AppSecret::generate();
        assert!(!format!("{s:?}").contains(s.as_str()));
        assert_eq!(format!("{s}"), "[REDACTED]");
    }
}
