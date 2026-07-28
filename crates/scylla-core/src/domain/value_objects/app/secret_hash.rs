use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;
use std::fmt;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("App secret hash cannot be empty"));
    }
    if !s.starts_with('$') {
        return Err(DomainError::validation(
            "App secret hash must be in PHC string format (starts with '$')",
        ));
    }
    Ok(())
}

/// Hashed App secret (PHC string format). Type-level guarantee that the wrapped
/// value is a hash, not the plaintext [`AppSecret`](crate::domain::value_objects::app::AppSecret). `Debug` /
/// `Display` are masked.
#[nutype(
    validate(with = validate, error = DomainError),
    derive(Clone, PartialEq, Eq, AsRef, Borrow, Into),
)]
pub struct AppSecretHash(String);

impl AppSecretHash {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        Self::try_new(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        <Self as AsRef<str>>::as_ref(self)
    }
}

impl fmt::Debug for AppSecretHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppSecretHash")
            .field("inner", &"[HASH]")
            .finish()
    }
}

impl fmt::Display for AppSecretHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[HASH]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_phc_hash() {
        assert!(AppSecretHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").is_ok());
    }

    #[test]
    fn rejects_plaintext_and_empty() {
        assert!(AppSecretHash::new("not-a-hash").is_err());
        assert!(AppSecretHash::new("").is_err());
    }

    #[test]
    fn display_masked() {
        let h = AppSecretHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap();
        assert_eq!(format!("{h}"), "[HASH]");
    }
}
