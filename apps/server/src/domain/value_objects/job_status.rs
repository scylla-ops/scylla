use crate::domain::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// JobStatus value object representing the state of a job execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is waiting to be executed
    Pending,
    /// Job is currently executing
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed during execution
    Failed,
    /// Job was cancelled before completion
    Cancelled,
}

impl JobStatus {
    /// Create a JobStatus from a string with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let trimmed = value.into().trim().to_lowercase();
        match trimmed.as_str() {
            "pending" => Ok(JobStatus::Pending),
            "running" => Ok(JobStatus::Running),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            "cancelled" => Ok(JobStatus::Cancelled),
            _ => Err(DomainError::validation(format!(
                "Invalid job status: {}",
                trimmed
            ))),
        }
    }

    /// Get the status as a string slice
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    }

    /// Check if the status is terminal (job execution finished)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }

    /// Check if transition to another status is allowed
    pub fn can_transition_to(&self, next: JobStatus) -> bool {
        match (self, &next) {
            // Pending can transition to Running or Cancelled
            (JobStatus::Pending, JobStatus::Running | JobStatus::Cancelled) => true,
            // Running can transition to Completed, Failed, or Cancelled
            (
                JobStatus::Running,
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled,
            ) => true,
            // Terminal states cannot transition anywhere
            (s, _) if s.is_terminal() => false,
            _ => false,
        }
    }

    /// Validate transition to new status, returning error if invalid
    pub fn validate_transition_to(&self, next: &JobStatus) -> DomainResult<()> {
        if !self.can_transition_to(*next) {
            return Err(DomainError::business_rule(format!(
                "Cannot transition job from {} to {}",
                self.as_str(),
                next.as_str()
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

impl From<JobStatus> for String {
    fn from(status: JobStatus) -> Self {
        status.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_creation_from_string() {
        assert!(JobStatus::new("pending").is_ok());
        assert!(JobStatus::new("running").is_ok());
        assert!(JobStatus::new("completed").is_ok());
        assert!(JobStatus::new("failed").is_ok());
        assert!(JobStatus::new("cancelled").is_ok());
        assert!(JobStatus::new("invalid").is_err());
    }

    #[test]
    fn test_job_status_case_insensitive() {
        assert_eq!(JobStatus::new("PENDING").unwrap(), JobStatus::Pending);
        assert_eq!(JobStatus::new("Running").unwrap(), JobStatus::Running);
        assert_eq!(JobStatus::new("COMPLETED").unwrap(), JobStatus::Completed);
    }

    #[test]
    fn test_is_terminal() {
        assert!(!JobStatus::Pending.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_valid_transitions() {
        // Pending -> Running | Cancelled
        assert!(JobStatus::Pending.can_transition_to(JobStatus::Running));
        assert!(JobStatus::Pending.can_transition_to(JobStatus::Cancelled));
        assert!(!JobStatus::Pending.can_transition_to(JobStatus::Completed));

        // Running -> Completed | Failed | Cancelled
        assert!(JobStatus::Running.can_transition_to(JobStatus::Completed));
        assert!(JobStatus::Running.can_transition_to(JobStatus::Failed));
        assert!(JobStatus::Running.can_transition_to(JobStatus::Cancelled));

        // Terminal states cannot transition
        assert!(!JobStatus::Completed.can_transition_to(JobStatus::Pending));
        assert!(!JobStatus::Failed.can_transition_to(JobStatus::Running));
        assert!(!JobStatus::Cancelled.can_transition_to(JobStatus::Completed));
    }
}
