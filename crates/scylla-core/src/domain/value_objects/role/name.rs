use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

const MAX_NAME_LENGTH: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct RoleName(String);

impl RoleName {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Role name cannot be empty"));
        }

        if trimmed.len() > MAX_NAME_LENGTH {
            return Err(DomainError::validation(format!(
                "Role name cannot exceed {MAX_NAME_LENGTH} characters"
            )));
        }

        Ok(Self(trimmed.to_string()))
    }

    #[must_use] 
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use] 
    pub fn into_string(self) -> String {
        self.0
    }

    #[must_use] 
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for RoleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for RoleName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
