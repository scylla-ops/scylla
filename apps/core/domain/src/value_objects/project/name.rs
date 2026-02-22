use crate::errors::{DomainError, DomainResult};
use std::fmt;

const MAX_NAME_LENGTH: usize = 255;

/// ProjectName value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
                "Project name cannot exceed {} characters",
                MAX_NAME_LENGTH
            )));
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

    pub fn len(&self) -> usize {
        self.inner.len()
    }

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

#[cfg(feature = "surrealdb")]
impl surrealdb_types::SurrealValue for ProjectName {
    fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::String
    }

    fn into_value(self) -> surrealdb_types::Value {
        surrealdb_types::Value::String(self.inner)
    }

    fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb_types::Error> {
        match value {
            surrealdb_types::Value::String(s) => Self::new(s).map_err(|e| {
                surrealdb_types::Error::internal(format!("Invalid ProjectName: {}", e))
            }),
            other => {
                Err(surrealdb_types::ConversionError::from_value(Self::kind_of(), &other).into())
            }
        }
    }
}
