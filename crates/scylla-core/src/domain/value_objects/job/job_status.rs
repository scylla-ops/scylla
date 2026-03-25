use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
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

    #[must_use] 
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Orphaned
        )
    }

    pub fn transition_to(&self, target: &JobStatus) -> DomainResult<()> {
        let valid = match self {
            Self::Pending => matches!(target, Self::Running | Self::Cancelled),
            Self::Running => {
                matches!(
                    target,
                    Self::Completed | Self::Failed | Self::Cancelled | Self::Orphaned
                )
            }
            Self::Completed | Self::Failed | Self::Cancelled | Self::Orphaned => false,
        };

        if !valid {
            return Err(DomainError::business_rule(format!(
                "Invalid status transition from {self} to {target}"
            )));
        }

        Ok(())
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
