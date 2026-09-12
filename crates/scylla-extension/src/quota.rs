//! Resource quotas: the one extension point wired today.
//!
//! The core asks the policy before creating a metered resource and turns a
//! [`QuotaDecision::Deny`] into a domain error. What a limit is, how usage is
//! counted and where the numbers come from are entirely the policy's business.

use async_trait::async_trait;
use std::fmt;

/// A kind of resource whose creation can be metered.
///
/// Scopes are organizations; this names what is being created inside one.
/// `non_exhaustive` so metering a new kind later is not a breaking change for
/// an out-of-tree policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Resource {
    Project,
}

impl Resource {
    /// Stable lowercase name, for logs and user-facing messages.
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

/// Outcome of a quota check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuotaDecision {
    /// One more resource may be created.
    Allow,
    /// The scope is at its limit.
    Deny {
        resource: Resource,
        /// The limit the scope is held to.
        limit: u64,
        /// How many the scope already has.
        current: u64,
        /// Free text a policy may attach to a refusal, typically how to raise
        /// the limit. Appended to the error message shown to the caller.
        upgrade_hint: Option<String>,
    },
}

/// Usage of one resource kind inside a scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaUsage {
    pub resource: Resource,
    /// How many the scope currently has.
    pub current: u64,
    /// The scope's limit; `None` when the policy imposes none.
    pub limit: Option<u64>,
}

/// The policy could not answer (a metering backend was unreachable, ...).
///
/// The core reports it as an infrastructure failure of the operation, never as
/// a decision: a policy that cannot answer neither allows nor denies.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct QuotaError(pub String);

/// Decides whether a scope may create one more resource.
///
/// `scope` is the organization id as a plain string, so the contract stays
/// free of the domain model's id newtypes (they are unvalidated `String`
/// wrappers, so the conversion is exact in both directions).
#[async_trait]
pub trait QuotaPolicy: Send + Sync {
    /// Whether `scope` may create one more `resource` right now.
    async fn check(&self, resource: Resource, scope: &str) -> Result<QuotaDecision, QuotaError>;

    /// Current usage of `resource` in `scope`, or `None` when the policy does
    /// not meter that resource for that scope.
    async fn usage(
        &self,
        resource: Resource,
        scope: &str,
    ) -> Result<Option<QuotaUsage>, QuotaError>;
}
