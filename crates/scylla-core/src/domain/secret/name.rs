use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

const MAX_SECRET_NAME_LENGTH: usize = 128;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("Secret name cannot be empty"));
    }
    if s.len() > MAX_SECRET_NAME_LENGTH {
        return Err(DomainError::validation(format!(
            "Secret name cannot exceed {MAX_SECRET_NAME_LENGTH} characters"
        )));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(DomainError::validation(
            "Secret name may only contain alphanumeric characters, '-', '_', and '.'",
        ));
    }
    Ok(())
}

/// A project-unique secret name (the handle an env var references). Stable,
/// human-chosen identifier — not the secret value.
#[nutype(
    sanitize(trim),
    validate(with = validate, error = DomainError),
    derive(
        Debug, Clone, PartialEq, Eq, Hash, AsRef, Borrow, Display, Into, Serialize, Deserialize,
    ),
)]
pub struct SecretName(String);

impl SecretName {
    /// Construct from anything string-like.
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
    fn accepts_valid_names() {
        for n in ["DB_PASSWORD", "api.token", "prod-key", "x"] {
            assert!(SecretName::new(n).is_ok(), "{n} should be valid");
        }
    }

    #[test]
    fn rejects_bad_names() {
        for n in ["", "with space", "weird$char", "a/b"] {
            assert!(SecretName::new(n).is_err(), "{n} should be rejected");
        }
    }
}
