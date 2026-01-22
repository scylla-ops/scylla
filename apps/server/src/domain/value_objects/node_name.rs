use crate::domain::errors::{DomainError, DomainResult};
use protocol::{Deserialize, Serialize};
use std::fmt;

/// NodeName value object - name of a node in a pipeline
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeName(String);

impl NodeName {
    /// Create a new NodeName with validation
    pub fn new(name: impl Into<String>) -> DomainResult<Self> {
        let name = name.into();
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("NodeName cannot be empty"));
        }

        if trimmed.len() > 255 {
            return Err(DomainError::validation(
                "NodeName cannot exceed 255 characters",
            ));
        }

        Ok(NodeName(trimmed.to_string()))
    }

    /// Get the NodeName as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<NodeName> for String {
    fn from(name: NodeName) -> Self {
        name.0
    }
}
