use chrono::{DateTime, Utc};

/// Outcome of an authorization decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditDecision {
    Allow,
    Deny,
}

impl AuditDecision {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

/// One authorization decision, captured for the persistent audit trail.
///
/// Holds the full context of a `PermissionService::check`: who acted, what they
/// tried, on which resource, the verdict, and — for forensics — the Cedar
/// policies that determined it.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub occurred_at: DateTime<Utc>,
    /// `user` | `service` | `anonymous`.
    pub principal_kind: &'static str,
    /// User id / service name; `None` for anonymous.
    pub principal_id: Option<String>,
    /// Cedar action eid, e.g. `runPipeline`.
    pub action: &'static str,
    /// `system` | `pipeline` | `job` | …
    pub resource_kind: &'static str,
    /// Resource id; `None` for the `system` singleton.
    pub resource_id: Option<String>,
    pub decision: AuditDecision,
    /// Ids of the Cedar policies that determined the decision (RBAC rule,
    /// ABAC rule, or a linked grant). Empty on a default-deny.
    pub policies: Vec<String>,
    /// Optional note — e.g. Cedar evaluation errors on a denial.
    pub reason: Option<String>,
}

/// Sink for the audit trail. Implementations must be cheap and non-blocking:
/// `record` is called inline on the authorization hot path, so the production
/// adapter enqueues and persists out-of-band.
pub trait AuditLog: Send + Sync {
    fn record(&self, entry: AuditEntry);
}

/// Discards entries. Used in tests and when persistence is disabled.
pub struct NoopAuditLog;

impl AuditLog for NoopAuditLog {
    fn record(&self, _entry: AuditEntry) {}
}
