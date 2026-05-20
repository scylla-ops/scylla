use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

/// Lifecycle of an invitation. Persisted as lowercase text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Revoked,
}

impl InvitationStatus {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        match value.as_str() {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "revoked" => Ok(Self::Revoked),
            other => Err(DomainError::validation(format!(
                "Invalid invitation status: {other}"
            ))),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Revoked => "revoked",
        }
    }
}

impl fmt::Display for InvitationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<str> for InvitationStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
