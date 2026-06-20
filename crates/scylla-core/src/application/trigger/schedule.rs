use crate::domain::errors::DomainResult;
use chrono::{DateTime, Utc};

/// Computes cron occurrences. Kept as an application port so the domain stays
/// free of a cron library: `CronSpec` validates only the 5-field *shape*, while
/// semantic parsing and next-occurrence math live in the infrastructure
/// implementation. All times are UTC (v0.3 has no per-tenant timezone).
pub trait CronSchedule: Send + Sync {
    /// The next occurrence strictly after `after`. Errors when the expression is
    /// syntactically 5-field-shaped but semantically invalid (e.g. `"99 * * * *"`),
    /// which the shape check at create time cannot catch.
    fn next_after(&self, expression: &str, after: DateTime<Utc>)
    -> DomainResult<DateTime<Utc>>;
}
