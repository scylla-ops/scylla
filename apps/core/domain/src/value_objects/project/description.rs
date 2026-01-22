use crate::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::fmt;

const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Description value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectDescription {
    inner: String,
}

impl ProjectDescription {
    /// Create a new Description with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.len() > MAX_DESCRIPTION_LENGTH {
            return Err(DomainError::validation(format!(
                "Description cannot exceed {} characters",
                MAX_DESCRIPTION_LENGTH
            )));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
    }

    /// Create an empty description (for None case)
    pub fn empty() -> Self {
        Self {
            inner: String::new(),
        }
    }

    /// Check if the description is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the description as a string slice
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to inner String
    pub fn into_string(self) -> String {
        self.inner
    }

    /// Get the length of the description
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl fmt::Display for ProjectDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for ProjectDescription {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
