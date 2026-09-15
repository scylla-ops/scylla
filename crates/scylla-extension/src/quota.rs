use async_trait::async_trait;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Resource {
    Project,
}

impl Resource {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
        }
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuotaDecision {
    Allow,
    Deny {
        resource: Resource,
        limit: u64,
        current: u64,
        upgrade_hint: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaUsage {
    pub resource: Resource,
    pub current: u64,
    pub limit: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct QuotaError(pub String);

/// `scope` is the organization id.
#[async_trait]
pub trait QuotaPolicy: Send + Sync {
    async fn check(&self, resource: Resource, scope: &str) -> Result<QuotaDecision, QuotaError>;

    async fn usage(
        &self,
        resource: Resource,
        scope: &str,
    ) -> Result<Option<QuotaUsage>, QuotaError>;
}
