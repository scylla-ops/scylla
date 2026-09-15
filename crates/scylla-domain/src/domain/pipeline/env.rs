use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::secret::SecretName;
use nutype::nutype;
use serde::{Deserialize, Serialize};

/// The agent injects its context vars under this prefix; user keys may not shadow them.
const RESERVED_PREFIX: &str = "SCYLLA_";

fn validate_key(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("Env var key cannot be empty"));
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(DomainError::validation(
            "Env var key must start with a letter or underscore",
        ));
    }
    if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(DomainError::validation(
            "Env var key may only contain letters, digits, and underscores",
        ));
    }
    if s.starts_with(RESERVED_PREFIX) {
        return Err(DomainError::validation(format!(
            "Env var key may not start with the reserved prefix `{RESERVED_PREFIX}`"
        )));
    }
    Ok(())
}

#[nutype(
    sanitize(trim),
    validate(with = validate_key, error = DomainError),
    derive(
        Debug, Clone, PartialEq, Eq, Hash, AsRef, Borrow, Display, Into, Serialize, Deserialize,
    ),
)]
pub struct EnvKey(String);

impl EnvKey {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        Self::try_new(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        <Self as AsRef<str>>::as_ref(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvSource {
    Literal(String),
    Secret(SecretName),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvVar {
    key: EnvKey,
    source: EnvSource,
}

impl EnvVar {
    #[must_use]
    pub fn literal(key: EnvKey, value: String) -> Self {
        Self {
            key,
            source: EnvSource::Literal(value),
        }
    }

    #[must_use]
    pub fn secret(key: EnvKey, secret: SecretName) -> Self {
        Self {
            key,
            source: EnvSource::Secret(secret),
        }
    }

    #[must_use]
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    #[must_use]
    pub fn source(&self) -> &EnvSource {
        &self.source
    }

    #[must_use]
    pub fn literal_value(&self) -> Option<&str> {
        match &self.source {
            EnvSource::Literal(v) => Some(v),
            EnvSource::Secret(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_keys() {
        for k in ["PATH", "MY_VAR", "_x", "a1_b2"] {
            assert!(EnvKey::new(k).is_ok(), "{k} should be valid");
        }
    }

    #[test]
    fn rejects_bad_keys() {
        for k in ["", "1ABC", "with-dash", "with space", "SCYLLA_JOB_ID"] {
            assert!(EnvKey::new(k).is_err(), "{k} should be rejected");
        }
    }
}
