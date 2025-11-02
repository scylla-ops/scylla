use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

/// JobContent value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JobContent {
    inner: String,
}

impl JobContent {
    /// Create a new JobContent with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Job content cannot be empty"));
        }

        Ok(Self {
            inner: value, // keep original value, not trimmed
        })
    }

    /// Get the content as a string slice
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to inner String
    pub fn into_string(self) -> String {
        self.inner
    }

    /// Get the length of the content
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the content is empty
    pub fn is_empty(&self) -> bool {
        self.inner.trim().is_empty()
    }

    /// Get the trimmed content
    pub fn trimmed(&self) -> &str {
        self.inner.trim()
    }
}

impl fmt::Display for JobContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for JobContent {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl PartialEq<str> for JobContent {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<&str> for JobContent {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_content_creation() {
        assert!(JobContent::new("valid content").is_ok());
        assert!(JobContent::new("  valid content  ").is_ok()); // whitespace allowed
        assert!(JobContent::new("").is_err()); // empty
        assert!(JobContent::new("   ").is_err()); // whitespace only
    }

    #[test]
    fn test_job_content_validation() {
        // Valid content
        assert!(JobContent::new("job script").is_ok());
        assert!(JobContent::new("a").is_ok());

        // Invalid content
        assert!(JobContent::new("").is_err());
        assert!(JobContent::new("   ").is_err());
        assert!(JobContent::new("\t\n  \t").is_err());
    }

    #[test]
    fn test_job_content_methods() {
        let content = JobContent::new("  my job  ").unwrap();
        assert_eq!(content.as_str(), "  my job  ");
        assert_eq!(content.trimmed(), "my job");
        assert!(!content.is_empty());
        assert_eq!(content.len(), 10);
    }

    #[test]
    fn test_job_content_comparison() {
        let content = JobContent::new("my job").unwrap();
        assert_eq!(content, "my job");
        assert_eq!(content.as_str(), "my job");
    }
}
