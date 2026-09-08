use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::pipeline::EnvKey;
use serde::{Deserialize, Serialize};

/// Where a [`TriggerInput`]'s value comes from. Externally tagged so the
/// persisted JSONB is self-describing: `{"literal":"prod"}` /
/// `{"json_pointer":"/after"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerInputSource {
    /// A constant value, usable by any source kind.
    Literal(String),
    /// An RFC 6901 JSON pointer into the webhook payload (webhook sources only).
    JsonPointer(String),
}

/// One run input contributed by a trigger, merged into the run as a literal
/// (unmasked) environment variable after secret resolution. The key reuses
/// [`EnvKey`], so it cannot shadow the reserved `SCYLLA_` namespace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerInput {
    key: EnvKey,
    source: TriggerInputSource,
}

impl TriggerInput {
    /// An input with a constant value.
    #[must_use]
    pub fn literal(key: EnvKey, value: impl Into<String>) -> Self {
        Self {
            key,
            source: TriggerInputSource::Literal(value.into()),
        }
    }

    /// An input extracted from the webhook payload via a JSON pointer. The
    /// pointer must be a non-empty RFC 6901 reference (i.e. start with `/`).
    pub fn json_pointer(key: EnvKey, pointer: impl Into<String>) -> DomainResult<Self> {
        let pointer = pointer.into();
        if pointer.is_empty() || !pointer.starts_with('/') {
            return Err(DomainError::validation(format!(
                "JSON pointer for input '{key}' must be non-empty and start with '/'"
            )));
        }
        Ok(Self {
            key,
            source: TriggerInputSource::JsonPointer(pointer),
        })
    }

    #[must_use]
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    #[must_use]
    pub fn source(&self) -> &TriggerInputSource {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(k: &str) -> EnvKey {
        EnvKey::new(k).unwrap()
    }

    #[test]
    fn json_pointer_requires_leading_slash() {
        assert!(TriggerInput::json_pointer(key("GIT_COMMIT"), "/after").is_ok());
        assert!(TriggerInput::json_pointer(key("GIT_COMMIT"), "after").is_err());
        assert!(TriggerInput::json_pointer(key("GIT_COMMIT"), "").is_err());
    }

    #[test]
    fn source_json_is_externally_tagged() {
        let lit = TriggerInput::literal(key("RUN_MODE"), "nightly");
        let json = serde_json::to_string(&lit).unwrap();
        assert!(json.contains(r#""literal":"nightly""#), "{json}");

        let ptr = TriggerInput::json_pointer(key("GIT_REF"), "/ref").unwrap();
        let json = serde_json::to_string(&ptr).unwrap();
        assert!(json.contains(r#""json_pointer":"/ref""#), "{json}");
    }
}
