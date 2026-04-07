use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Description value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct ProjectDescription {
    inner: String,
}

impl ProjectDescription {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.len() > MAX_DESCRIPTION_LENGTH {
            return Err(DomainError::validation(format!(
                "Description cannot exceed {MAX_DESCRIPTION_LENGTH} characters"
            )));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
    }

    /// Create an empty description (for None case)
    #[must_use]
    pub fn empty() -> Self {
        Self {
            inner: String::new(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.inner
    }

    #[must_use]
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
