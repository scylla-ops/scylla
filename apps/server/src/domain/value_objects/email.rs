use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

/// Email value object with basic validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email {
    inner: String,
}

impl Email {
    /// Create a new Email with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim().to_lowercase();

        if !Self::is_valid_email(&trimmed) {
            return Err(DomainError::validation(format!(
                "Invalid email format: {}",
                value
            )));
        }

        Ok(Self { inner: trimmed })
    }

    /// Email validation
    fn is_valid_email(email: &str) -> bool {
        if email.is_empty() {
            return false;
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let local = parts[0];
        let domain = parts[1];

        // Check local part
        if local.is_empty() || local.len() > 64 {
            return false;
        }

        // Check domain part
        if domain.is_empty() || domain.len() > 255 {
            return false;
        }

        // Domain must contain at least one dot
        if !domain.contains('.') {
            return false;
        }

        // Domain parts cannot be empty
        if domain.split('.').any(|part| part.is_empty()) {
            return false;
        }

        true
    }

    /// Get the email as a string slice
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to inner String
    pub fn into_string(self) -> String {
        self.inner
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let email = Email::new("user@example.com");
        assert!(email.is_ok());
        assert_eq!(email.unwrap().as_str(), "user@example.com");
    }

    #[test]
    fn test_email_lowercase() {
        let email = Email::new("USER@EXAMPLE.COM").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_email_trims_whitespace() {
        let email = Email::new("  user@example.com  ").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_invalid_email_no_at() {
        let email = Email::new("userexample.com");
        assert!(email.is_err());
    }

    #[test]
    fn test_invalid_email_multiple_at() {
        let email = Email::new("user@@example.com");
        assert!(email.is_err());
    }

    #[test]
    fn test_invalid_email_no_domain() {
        let email = Email::new("user@");
        assert!(email.is_err());
    }

    #[test]
    fn test_invalid_email_no_local() {
        let email = Email::new("@example.com");
        assert!(email.is_err());
    }

    #[test]
    fn test_invalid_email_no_dot_in_domain() {
        let email = Email::new("user@example");
        assert!(email.is_err());
    }

    #[test]
    fn test_invalid_email_empty() {
        let email = Email::new("");
        assert!(email.is_err());
    }
}
