use chrono::{DateTime, SubsecRound, Utc};

/// Current UTC time truncated to microsecond precision.
///
/// Postgres `TIMESTAMPTZ` stores microseconds, so generating timestamps at this
/// resolution keeps an in-memory entity's `created_at` / `updated_at` exactly
/// equal to the value read back after a database round-trip. chrono's native
/// nanosecond precision would otherwise be silently truncated on write, breaking
/// equality between a freshly created entity and its persisted form.
#[must_use]
pub fn now() -> DateTime<Utc> {
    Utc::now().trunc_subsecs(6)
}
