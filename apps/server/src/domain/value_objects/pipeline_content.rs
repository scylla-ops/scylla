use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

/// PipelineContent value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineContent {
    inner: String,
}

impl PipelineContent {
    /// Create a new PipelineContent with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Pipeline content cannot be empty"));
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

impl fmt::Display for PipelineContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for PipelineContent {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl PartialEq<str> for PipelineContent {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<&str> for PipelineContent {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_content_creation() {
        assert!(PipelineContent::new("valid content").is_ok());
        assert!(PipelineContent::new("  valid content  ").is_ok()); // whitespace allowed
        assert!(PipelineContent::new("").is_err()); // empty
        assert!(PipelineContent::new("   ").is_err()); // whitespace only
    }

    #[test]
    fn test_pipeline_content_validation() {
        // Valid content
        assert!(PipelineContent::new("pipeline script").is_ok());
        assert!(PipelineContent::new("a").is_ok());

        // Invalid content
        assert!(PipelineContent::new("").is_err());
        assert!(PipelineContent::new("   ").is_err());
        assert!(PipelineContent::new("\t\n  \t").is_err());
    }

    #[test]
    fn test_pipeline_content_methods() {
        let content = PipelineContent::new("  my pipeline  ").unwrap();
        assert_eq!(content.as_str(), "  my pipeline  ");
        assert_eq!(content.trimmed(), "my pipeline");
        assert!(!content.is_empty());
        assert_eq!(content.len(), 15);
    }

    #[test]
    fn test_pipeline_content_comparison() {
        let content = PipelineContent::new("my pipeline").unwrap();
        assert_eq!(content, "my pipeline");
        assert_eq!(content.as_str(), "my pipeline");
    }
}
