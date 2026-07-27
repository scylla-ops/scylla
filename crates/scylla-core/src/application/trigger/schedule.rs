use crate::domain::entities::Trigger;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::trigger::TriggerSource;
use chrono::{DateTime, Utc};

/// Computes cron occurrences. Kept as an application port so the domain stays
/// free of a cron library: `CronSpec` validates only the 5-field *shape*, while
/// semantic parsing and next-occurrence math live in the infrastructure
/// implementation. All times are UTC (v0.3 has no per-tenant timezone).
pub trait CronSchedule: Send + Sync {
    /// The next occurrence strictly after `after`. Errors when the expression is
    /// syntactically 5-field-shaped but semantically invalid (e.g. `"99 * * * *"`),
    /// which the shape check at create time cannot catch.
    fn next_after(&self, expression: &str, after: DateTime<Utc>) -> DomainResult<DateTime<Utc>>;
}

/// THE single source of a trigger's schedule timing: the cron's next occurrence
/// strictly after `now`, or `None` for a webhook (push-driven, never scheduled).
/// Every path that (re)anchors `next_fire_at` — create, update, re-enable, the
/// scheduler's seed, and the scheduler's claim/advance — routes through here, so
/// there is exactly one rescheduling rule. Errors propagate a semantically
/// invalid cron expression (e.g. `"99 * * * *"`) to the caller, giving create /
/// update validation for free.
pub fn next_fire_time(
    trigger: &Trigger,
    schedule: &dyn CronSchedule,
    now: DateTime<Utc>,
) -> DomainResult<Option<DateTime<Utc>>> {
    match trigger.source() {
        TriggerSource::Cron(spec) => schedule.next_after(spec.expression(), now).map(Some),
        TriggerSource::Webhook(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::PipelineId;
    use crate::domain::errors::DomainError;
    use crate::domain::value_objects::trigger::{CronSpec, TriggerName, WebhookSpec};

    /// Fixed +1h for any expression, except one starting with `99` (semantically
    /// invalid but 5-field-shaped) which it rejects like the real service would.
    struct StubSchedule;
    impl CronSchedule for StubSchedule {
        fn next_after(&self, expr: &str, after: DateTime<Utc>) -> DomainResult<DateTime<Utc>> {
            if expr.starts_with("99") {
                return Err(DomainError::validation("out of range"));
            }
            Ok(after + chrono::Duration::hours(1))
        }
    }

    fn cron(expr: &str) -> Trigger {
        Trigger::create(
            PipelineId::new("p"),
            TriggerName::new("t").unwrap(),
            TriggerSource::Cron(CronSpec::new(expr).unwrap()),
            vec![],
        )
        .unwrap()
    }

    fn now() -> DateTime<Utc> {
        crate::domain::clock::now()
    }

    #[test]
    fn cron_returns_next_occurrence() {
        let n = now();
        assert_eq!(
            next_fire_time(&cron("0 9 * * *"), &StubSchedule, n).unwrap(),
            Some(n + chrono::Duration::hours(1)),
        );
    }

    #[test]
    fn webhook_has_no_schedule() {
        let hook = Trigger::create(
            PipelineId::new("p"),
            TriggerName::new("hook").unwrap(),
            TriggerSource::Webhook(WebhookSpec::new(None).unwrap()),
            vec![],
        )
        .unwrap();
        assert_eq!(next_fire_time(&hook, &StubSchedule, now()).unwrap(), None);
    }

    #[test]
    fn invalid_expression_propagates_error() {
        // 5-field shape passes CronSpec, but the schedule rejects it → caller sees
        // the error (this is what gives create/update validation).
        assert!(next_fire_time(&cron("99 * * * *"), &StubSchedule, now()).is_err());
    }
}
