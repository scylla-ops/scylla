use crate::errors::{DomainError, DomainResult};
use std::fmt;

const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Description value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectDescription {
    inner: String,
}

impl ProjectDescription {
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

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
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

#[cfg(feature = "surrealdb")]
impl surrealdb_types::SurrealValue for ProjectDescription {
    fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::Either(vec![
            surrealdb_types::Kind::String,
            surrealdb_types::Kind::None,
        ])
    }

    fn into_value(self) -> surrealdb_types::Value {
        if self.inner.is_empty() {
            surrealdb_types::Value::None
        } else {
            surrealdb_types::Value::String(self.inner)
        }
    }

    fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb_types::Error> {
        match value {
            surrealdb_types::Value::String(s) => Self::new(s).map_err(|e| {
                surrealdb_types::Error::internal(format!("Invalid ProjectDescription: {}", e))
            }),
            surrealdb_types::Value::None | surrealdb_types::Value::Null => Ok(Self::empty()),
            other => {
                Err(surrealdb_types::ConversionError::from_value(Self::kind_of(), &other).into())
            }
        }
    }
}
