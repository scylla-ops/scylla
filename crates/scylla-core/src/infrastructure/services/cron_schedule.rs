use crate::application::CronSchedule;
use crate::domain::errors::{DomainError, DomainResult};
use chrono::{DateTime, Utc};
use croner::Cron;
use std::str::FromStr;

/// [`CronSchedule`] backed by the `croner` crate: standard 5-field Vixie cron
/// (`min hour dom mon dow`), evaluated in UTC. This is the one place the cron
/// library is allowed; the domain and application layers stay library-free.
#[derive(Debug, Default, Clone)]
pub struct CronScheduleService;

impl CronScheduleService {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl CronSchedule for CronScheduleService {
    fn next_after(
        &self,
        expression: &str,
        after: DateTime<Utc>,
    ) -> DomainResult<DateTime<Utc>> {
        let cron = Cron::from_str(expression).map_err(|e| {
            DomainError::validation(format!("invalid cron expression '{expression}': {e}"))
        })?;
        // inclusive = false → strictly after `after`, so a trigger never re-fires
        // the same occurrence it was just claimed at.
        cron.find_next_occurrence(&after, false).map_err(|e| {
            DomainError::validation(format!(
                "no upcoming occurrence for cron '{expression}': {e}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, s).unwrap()
    }

    #[test]
    fn computes_next_daily_occurrence() {
        let svc = CronScheduleService::new();
        let next = svc.next_after("0 9 * * *", at(2026, 1, 1, 8, 0, 0)).unwrap();
        assert_eq!(next, at(2026, 1, 1, 9, 0, 0));
    }

    #[test]
    fn occurrence_is_strictly_after_the_reference() {
        let svc = CronScheduleService::new();
        // Exactly on the boundary with inclusive=false rolls to the next day.
        let next = svc.next_after("0 9 * * *", at(2026, 1, 1, 9, 0, 0)).unwrap();
        assert_eq!(next, at(2026, 1, 2, 9, 0, 0));
    }

    #[test]
    fn every_minute_advances_to_the_next_minute_boundary() {
        let svc = CronScheduleService::new();
        let next = svc.next_after("* * * * *", at(2026, 1, 1, 8, 0, 30)).unwrap();
        assert_eq!(next, at(2026, 1, 1, 8, 1, 0));
    }

    #[test]
    fn rejects_semantically_invalid_but_five_field_expression() {
        let svc = CronScheduleService::new();
        // Passes the domain's 5-field shape check but is out of range.
        assert!(svc.next_after("99 * * * *", at(2026, 1, 1, 0, 0, 0)).is_err());
    }
}
