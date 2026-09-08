use crate::domain::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Orphaned,
}

impl JobStatus {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim().to_lowercase();

        match trimmed.as_str() {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "orphaned" => Ok(Self::Orphaned),
            _ => Err(DomainError::validation(format!(
                "Invalid job status: {value}"
            ))),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Orphaned => "orphaned",
        }
    }
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<str> for JobStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
