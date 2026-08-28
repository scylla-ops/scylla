use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;
use std::fmt;

pub const PASSWORD_MIN_LENGTH: usize = 8;
pub const PASSWORD_MAX_LENGTH: usize = 255;

fn validate(s: &str) -> Result<(), DomainError> {
    let len = s.chars().count();
    if len == 0 {
        return Err(DomainError::validation("Password cannot be empty"));
    }
    if s.trim().is_empty() {
        return Err(DomainError::validation(
            "Password cannot be whitespace-only",
        ));
    }
    if len < PASSWORD_MIN_LENGTH {
        return Err(DomainError::validation(format!(
            "Password too short, minimum length is {PASSWORD_MIN_LENGTH}"
        )));
    }
    if len > PASSWORD_MAX_LENGTH {
        return Err(DomainError::validation(format!(
            "Password too long, maximum length is {PASSWORD_MAX_LENGTH}"
        )));
    }
    Ok(())
}

/// Plaintext password held only during creation/validation.
/// `Debug` and `Display` are masked so the value never leaks into logs.
#[nutype(
    validate(with = validate, error = DomainError),
    derive(Clone, PartialEq, Eq, AsRef, Borrow, Into),
)]
pub struct Password(String);

impl Password {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        Self::try_new(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        <Self as AsRef<str>>::as_ref(self)
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Password")
            .field("inner", &"[REDACTED]")
            .finish()
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let masked = "*".repeat(self.as_str().chars().count());
        f.write_str(&masked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        assert!(Password::new("SecurePass123!").is_ok());
    }

    #[test]
    fn test_password_too_short() {
        assert!(Password::new("short").is_err());
    }

    #[test]
    fn test_password_empty() {
        assert!(Password::new("").is_err());
    }

    #[test]
    fn test_password_whitespace_only() {
        assert!(Password::new("        ").is_err());
    }

    #[test]
    fn test_password_display_hides_content() {
        let password = Password::new("SecurePass123!").unwrap();
        let display = format!("{password}");
        assert!(!display.contains("SecurePass"));
        assert!(display.contains('*'));
    }

    #[test]
    fn test_password_debug_redacted() {
        let password = Password::new("SecurePass123!").unwrap();
        let debug = format!("{password:?}");
        assert!(!debug.contains("SecurePass"));
        assert!(debug.contains("[REDACTED]"));
    }

    #[test]
    fn test_password_too_long() {
        assert!(Password::new("a".repeat(256)).is_err());
    }
}
