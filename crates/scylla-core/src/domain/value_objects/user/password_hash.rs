use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;
use std::fmt;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("Password hash cannot be empty"));
    }
    if !s.starts_with('$') {
        return Err(DomainError::validation(
            "Password hash must be in PHC string format (starts with '$')",
        ));
    }
    Ok(())
}

/// Value object representing a hashed password (PHC string format).
/// Guarantees at the type level that the wrapped value is a hash, not plaintext.
/// `Debug` and `Display` are masked.
#[nutype(
    validate(with = validate, error = DomainError),
    derive(Clone, PartialEq, Eq, AsRef, Borrow, Into),
)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        Self::try_new(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        <Self as AsRef<str>>::as_ref(self)
    }
}

impl fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PasswordHash")
            .field("inner", &"[HASH]")
            .finish()
    }
}

impl fmt::Display for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[HASH]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_argon2_hash() {
        let hash = PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def");
        assert!(hash.is_ok());
    }

    #[test]
    fn test_valid_bcrypt_hash() {
        let hash = PasswordHash::new("$2b$12$abcdefghijklmnopqrstuuABCDEFGHIJKLMNOPQRSTUVWXYZ01");
        assert!(hash.is_ok());
    }

    #[test]
    fn test_empty_hash_rejected() {
        assert!(PasswordHash::new("").is_err());
    }

    #[test]
    fn test_plaintext_rejected() {
        assert!(PasswordHash::new("my_password_123").is_err());
    }

    #[test]
    fn test_display_masked() {
        let hash = PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap();
        assert_eq!(format!("{hash}"), "[HASH]");
    }

    #[test]
    fn test_debug_masked() {
        let hash = PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap();
        let debug = format!("{hash:?}");
        assert!(!debug.contains("argon2"));
        assert!(debug.contains("[HASH]"));
    }

    #[test]
    fn test_as_str() {
        let raw = "$argon2id$v=19$m=19456,t=2,p=1$abc$def";
        let hash = PasswordHash::new(raw).unwrap();
        assert_eq!(hash.as_str(), raw);
    }
}
