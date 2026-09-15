use chrono::{DateTime, Utc};

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

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub occurred_at: DateTime<Utc>,
    pub principal_kind: &'static str,
    pub principal_id: Option<String>,
    pub action: &'static str,
    pub resource_kind: &'static str,
    pub resource_id: Option<String>,
    pub decision: AuditDecision,
    pub policies: Vec<String>,
    pub reason: Option<String>,
}

/// `record` runs inline on the authorization path: implementations must not block.
pub trait AuditLog: Send + Sync {
    fn record(&self, entry: AuditEntry);
}

pub struct NoopAuditLog;

impl AuditLog for NoopAuditLog {
    fn record(&self, _entry: AuditEntry) {}
}
