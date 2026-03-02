use crate::errors::{DomainError, DomainResult};
use std::fmt;
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

const MAX_NODE_ID_LENGTH: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct NodeId {
    inner: String,
}

impl NodeId {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Node ID cannot be empty"));
        }

        if trimmed.len() > MAX_NODE_ID_LENGTH {
            return Err(DomainError::validation(format!(
                "Node ID cannot exceed {} characters",
                MAX_NODE_ID_LENGTH
            )));
        }

        if !trimmed
            .chars()
            .all(|c| (c.is_ascii_alphanumeric() && !c.is_ascii_uppercase()) || c == '-' || c == '_')
        {
            return Err(DomainError::validation(
                "Node ID may only contain lowercase alphanumeric characters, hyphens, and underscores",
            ));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn into_string(self) -> String {
        self.inner
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for NodeId {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
