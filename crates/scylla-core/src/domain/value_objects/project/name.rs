use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

const MAX_NAME_LENGTH: usize = 255;

/// `ProjectName` value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct ProjectName {
    inner: String,
}

impl ProjectName {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Project name cannot be empty"));
        }

        if trimmed.len() > MAX_NAME_LENGTH {
            return Err(DomainError::validation(format!(
                "Project name cannot exceed {MAX_NAME_LENGTH} characters"
            )));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
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

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl fmt::Display for ProjectName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for ProjectName {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
