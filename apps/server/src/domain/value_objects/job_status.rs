use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

/// JobStatus value object with validation and business rules
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JobStatus {
    inner: JobStatusInner,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum JobStatusInner {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    /// Create a new JobStatus from string with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim().to_lowercase();

        let inner = match trimmed.as_str() {
            "pending" => JobStatusInner::Pending,
            "running" => JobStatusInner::Running,
            "completed" => JobStatusInner::Completed,
            "failed" => JobStatusInner::Failed,
            "cancelled" => JobStatusInner::Cancelled,
            _ => {
                return Err(DomainError::validation(format!(
                    "Invalid job status: {}",
                    value
                )));
            }
        };

        Ok(Self { inner })
    }

    /// Create a pending job status
    pub fn pending() -> Self {
        Self {
            inner: JobStatusInner::Pending,
        }
    }

    /// Create a running job status
    pub fn running() -> Self {
        Self {
            inner: JobStatusInner::Running,
        }
    }

    /// Create a completed job status
    pub fn completed() -> Self {
        Self {
            inner: JobStatusInner::Completed,
        }
    }

    /// Create a failed job status
    pub fn failed() -> Self {
        Self {
            inner: JobStatusInner::Failed,
        }
    }

    /// Create a cancelled job status
    pub fn cancelled() -> Self {
        Self {
            inner: JobStatusInner::Cancelled,
        }
    }

    /// Get the status as a string slice
    pub fn as_str(&self) -> &'static str {
        match self.inner {
            JobStatusInner::Pending => "pending",
            JobStatusInner::Running => "running",
            JobStatusInner::Completed => "completed",
            JobStatusInner::Failed => "failed",
            JobStatusInner::Cancelled => "cancelled",
        }
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    /// Check if the status is terminal (completed, failed, or cancelled)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.inner,
            JobStatusInner::Completed | JobStatusInner::Failed | JobStatusInner::Cancelled
        )
    }

    /// Check if the status can transition to another status
    pub fn can_transition_to(&self, new_status: &JobStatus) -> bool {
        match (&self.inner, &new_status.inner) {
            // Pending can go to Running or Cancelled
            (JobStatusInner::Pending, JobStatusInner::Running) => true,
            (JobStatusInner::Pending, JobStatusInner::Cancelled) => true,

            // Running can go to Completed, Failed, or Cancelled
            (JobStatusInner::Running, JobStatusInner::Completed) => true,
            (JobStatusInner::Running, JobStatusInner::Failed) => true,
            (JobStatusInner::Running, JobStatusInner::Cancelled) => true,

            // Terminal states cannot transition
            (JobStatusInner::Completed, _) => false,
            (JobStatusInner::Failed, _) => false,
            (JobStatusInner::Cancelled, _) => false,

            _ => false,
        }
    }

    /// Validate transition to new status
    pub fn validate_transition_to(&self, new_status: &JobStatus) -> DomainResult<()> {
        if !self.can_transition_to(new_status) {
            return Err(DomainError::business_rule(format!(
                "Cannot transition job from {} to {}",
                self.as_str(),
                new_status.as_str()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_creation() {
        assert!(JobStatus::new("pending").is_ok());
        assert!(JobStatus::new("running").is_ok());
        assert!(JobStatus::new("completed").is_ok());
        assert!(JobStatus::new("failed").is_ok());
        assert!(JobStatus::new("cancelled").is_ok());
        assert!(JobStatus::new("invalid").is_err());
    }

    #[test]
    fn test_job_status_case_insensitive() {
        assert_eq!(JobStatus::new("PENDING").unwrap().as_str(), "pending");
        assert_eq!(JobStatus::new("Running").unwrap().as_str(), "running");
        assert_eq!(JobStatus::new("COMPLETED").unwrap().as_str(), "completed");
    }

    #[test]
    fn test_job_status_terminal() {
        assert!(!JobStatus::pending().is_terminal());
        assert!(!JobStatus::running().is_terminal());
        assert!(JobStatus::completed().is_terminal());
        assert!(JobStatus::failed().is_terminal());
        assert!(JobStatus::cancelled().is_terminal());
    }

    #[test]
    fn test_job_status_transitions() {
        let pending = JobStatus::pending();
        let running = JobStatus::running();
        let completed = JobStatus::completed();
        let failed = JobStatus::failed();
        let cancelled = JobStatus::cancelled();

        // Pending transitions
        assert!(pending.can_transition_to(&running));
        assert!(pending.can_transition_to(&cancelled));
        assert!(!pending.can_transition_to(&completed));
        assert!(!pending.can_transition_to(&failed));

        // Running transitions
        assert!(running.can_transition_to(&completed));
        assert!(running.can_transition_to(&failed));
        assert!(running.can_transition_to(&cancelled));
        assert!(!running.can_transition_to(&pending));

        // Terminal states cannot transition
        assert!(!completed.can_transition_to(&pending));
        assert!(!completed.can_transition_to(&running));
        assert!(!failed.can_transition_to(&pending));
        assert!(!cancelled.can_transition_to(&pending));
    }
}
