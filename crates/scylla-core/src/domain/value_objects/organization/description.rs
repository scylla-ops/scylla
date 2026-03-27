use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Description value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrganizationDescription {
    inner: String,
}

impl OrganizationDescription {
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

impl fmt::Display for OrganizationDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for OrganizationDescription {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl PartialEq<str> for OrganizationDescription {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<&str> for OrganizationDescription {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

impl From<Option<String>> for OrganizationDescription {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(desc) => Self::new(desc).unwrap_or_else(|_| Self::empty()),
            None => Self::empty(),
        }
    }
}

impl From<OrganizationDescription> for Option<String> {
    fn from(value: OrganizationDescription) -> Self {
        if value.is_empty() {
            None
        } else {
            Some(value.into_string())
        }
    }
}

#[cfg(feature = "surrealdb")]
impl surrealdb_types::SurrealValue for OrganizationDescription {
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
                surrealdb_types::Error::internal(format!("Invalid OrganizationDescription: {e}"))
            }),
            surrealdb_types::Value::None | surrealdb_types::Value::Null => Ok(Self::empty()),
            other => {
                Err(surrealdb_types::ConversionError::from_value(Self::kind_of(), &other).into())
            }
        }
    }
}
