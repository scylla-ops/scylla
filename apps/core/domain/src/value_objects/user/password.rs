use crate::errors::{DomainError, DomainResult};
use std::fmt;

pub const PASSWORD_MIN_LENGTH: usize = 8;
pub const PASSWORD_MAX_LENGTH: usize = 255;

/// Password value object with domain validation rules
///
/// This is for plaintext passwords during creation/validation.
/// Stored passwords should be hashed using the PasswordHasher port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Password {
    inner: String,
}

impl Password {
    /// Create a new Password with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let len = value.chars().count();

        if len == 0 {
            return Err(DomainError::validation("Password cannot be empty"));
        }
        if value.trim().is_empty() {
            return Err(DomainError::validation(
                "Password cannot be whitespace-only",
            ));
        }
        if len < PASSWORD_MIN_LENGTH {
            return Err(DomainError::validation(format!(
                "Password too short, minimum length is {}",
                PASSWORD_MIN_LENGTH
            )));
        }
        if len > PASSWORD_MAX_LENGTH {
            return Err(DomainError::validation(format!(
                "Password too long, maximum length is {}",
                PASSWORD_MAX_LENGTH
            )));
        }

        Ok(Self { inner: value })
    }

    /// Get the password as a string slice
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to inner String
    pub fn into_string(self) -> String {
        self.inner
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never display the actual password, always show asterisks
        write!(f, "{}", "*".repeat(self.inner.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        let password = Password::new("SecurePass123!");
        assert!(password.is_ok());
    }

    #[test]
    fn test_password_too_short() {
        let password = Password::new("short");
        assert!(password.is_err());
    }

    #[test]
    fn test_password_empty() {
        let password = Password::new("");
        assert!(password.is_err());
    }

    #[test]
    fn test_password_whitespace_only() {
        let password = Password::new("        ");
        assert!(password.is_err());
    }

    #[test]
    fn test_password_display_hides_content() {
        let password = Password::new("SecurePass123!").unwrap();
        let display = format!("{}", password);
        assert!(!display.contains("SecurePass"));
        assert!(display.contains("*"));
    }

    #[test]
    fn test_password_too_long() {
        let long_pass = "a".repeat(256);
        let password = Password::new(long_pass);
        assert!(password.is_err());
    }
}
