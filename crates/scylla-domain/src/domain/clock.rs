use chrono::{DateTime, SubsecRound, Utc};

/// Microsecond precision: Postgres TIMESTAMPTZ truncates, so a round trip stays equal.
#[must_use]
pub fn now() -> DateTime<Utc> {
    Utc::now().trunc_subsecs(6)
}
