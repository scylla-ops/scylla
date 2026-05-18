use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

const MAX_USERNAME_LENGTH: usize = 255;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("Username cannot be empty"));
    }
    if s.len() > MAX_USERNAME_LENGTH {
        return Err(DomainError::validation(format!(
            "Username cannot exceed {MAX_USERNAME_LENGTH} characters"
        )));
    }
    Ok(())
}

#[nutype(
    sanitize(trim),
    validate(with = validate, error = DomainError),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Borrow, Display, Into),
)]
pub struct Username(String);

impl Username {
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
    fn validation_and_trimming() {
        assert!(Username::new("valid_user").is_ok());
        let trimmed = Username::new("  myuser  ").unwrap();
        assert_eq!(trimmed.as_str(), "myuser");
        assert!(Username::new("").is_err());
        assert!(Username::new("   ").is_err());
    }

    #[test]
    fn length_bounds() {
        assert!(Username::new("A").is_ok());
        assert!(Username::new("a".repeat(MAX_USERNAME_LENGTH)).is_ok());
        assert!(Username::new("a".repeat(MAX_USERNAME_LENGTH + 1)).is_err());
    }
}
