use async_trait::async_trait;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Resource {
    Project,
    Pipeline,
    Agent,
    Secret,
    Trigger,
}

/// The container a resource is counted in; the `scope` of a check is its id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScopeKind {
    Organization,
    Project,
    Pipeline,
}

impl Resource {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Pipeline => "pipeline",
            Self::Agent => "agent",
            Self::Secret => "secret",
            Self::Trigger => "trigger",
        }
    }

    #[must_use]
    pub fn scope_kind(self) -> ScopeKind {
        match self {
            Self::Project | Self::Agent => ScopeKind::Organization,
            Self::Pipeline | Self::Secret => ScopeKind::Project,
            Self::Trigger => ScopeKind::Pipeline,
        }
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ScopeKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Organization => "organization",
            Self::Project => "project",
            Self::Pipeline => "pipeline",
        }
    }
}

impl fmt::Display for ScopeKind {
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

/// `scope` is the id of the container `resource.scope_kind()` names: the
/// organization for a project or an agent, the project for a pipeline or a
/// secret, the pipeline for a trigger.
#[async_trait]
pub trait QuotaPolicy: Send + Sync {
    async fn check(&self, resource: Resource, scope: &str) -> Result<QuotaDecision, QuotaError>;

    async fn usage(
        &self,
        resource: Resource,
        scope: &str,
    ) -> Result<Option<QuotaUsage>, QuotaError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_resource_names_its_scope() {
        assert_eq!(Resource::Project.scope_kind(), ScopeKind::Organization);
        assert_eq!(Resource::Agent.scope_kind(), ScopeKind::Organization);
        assert_eq!(Resource::Pipeline.scope_kind(), ScopeKind::Project);
        assert_eq!(Resource::Secret.scope_kind(), ScopeKind::Project);
        assert_eq!(Resource::Trigger.scope_kind(), ScopeKind::Pipeline);
    }

    #[test]
    fn display_is_the_lowercase_name() {
        assert_eq!(Resource::Trigger.to_string(), "trigger");
        assert_eq!(ScopeKind::Organization.to_string(), "organization");
    }
}
