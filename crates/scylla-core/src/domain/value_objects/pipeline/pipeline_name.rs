use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

const MAX_PIPELINE_NAME_LENGTH: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct PipelineName {
    inner: String,
}

impl PipelineName {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Pipeline name cannot be empty"));
        }

        if trimmed.len() > MAX_PIPELINE_NAME_LENGTH {
            return Err(DomainError::validation(format!(
                "Pipeline name cannot exceed {MAX_PIPELINE_NAME_LENGTH} characters"
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
}

impl fmt::Display for PipelineName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for PipelineName {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
