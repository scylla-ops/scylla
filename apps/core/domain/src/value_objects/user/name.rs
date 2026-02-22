use crate::errors::{DomainError, DomainResult};
use std::fmt;

const MAX_USERNAME_LENGTH: usize = 255;

/// Username value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserName {
    inner: String,
}

impl UserName {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Username cannot be empty"));
        }

        if trimmed.len() > MAX_USERNAME_LENGTH {
            return Err(DomainError::validation(format!(
                "Username cannot exceed {} characters",
                MAX_USERNAME_LENGTH
            )));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
    }
}

impl UserName {
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

impl fmt::Display for UserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for UserName {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl PartialEq<str> for UserName {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<&str> for UserName {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

#[cfg(feature = "surrealdb")]
impl surrealdb_types::SurrealValue for UserName {
    fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::String
    }

    fn into_value(self) -> surrealdb_types::Value {
        surrealdb_types::Value::String(self.inner)
    }

    fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb_types::Error> {
        match value {
            surrealdb_types::Value::String(s) => Self::new(s)
                .map_err(|e| surrealdb_types::Error::internal(format!("Invalid UserName: {}", e))),
            other => {
                Err(surrealdb_types::ConversionError::from_value(Self::kind_of(), &other).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_creation() {
        assert!(UserName::new("valid_user").is_ok());
        assert!(UserName::new("  valid_user  ").is_ok());
        assert!(UserName::new("").is_err());
        assert!(UserName::new("   ").is_err());
    }

    #[test]
    fn test_username_validation() {
        assert!(UserName::new("user123").is_ok());
        assert!(UserName::new("A").is_ok());
        assert!(UserName::new("").is_err());
        assert!(UserName::new("   ").is_err());

        let long_username = "a".repeat(MAX_USERNAME_LENGTH + 1);
        assert!(UserName::new(long_username).is_err());

        let max_username = "a".repeat(MAX_USERNAME_LENGTH);
        assert!(UserName::new(max_username).is_ok());
    }

    #[test]
    fn test_username_trimming() {
        let username = UserName::new("  myuser  ").unwrap();
        assert_eq!(username.as_str(), "myuser");
    }

    #[test]
    fn test_username_comparison() {
        let username = UserName::new("myuser").unwrap();
        assert_eq!(username, "myuser");
        assert_eq!(username.as_str(), "myuser");
    }
}
