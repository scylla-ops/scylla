use crate::domain::errors::DomainResult;
use crate::domain::trigger::Trigger;
use crate::domain::trigger::TriggerSource;
use chrono::{DateTime, Utc};

pub trait CronSchedule: Send + Sync {
    fn next_after(&self, expression: &str, after: DateTime<Utc>) -> DomainResult<DateTime<Utc>>;
}

/// The single rescheduling rule: create, update, re-enable, seed and claim all route here.
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
    use crate::domain::errors::DomainError;
    use crate::domain::ids::PipelineId;
    use crate::domain::trigger::{CronSpec, TriggerName, WebhookSpec};

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
        assert!(next_fire_time(&cron("99 * * * *"), &StubSchedule, now()).is_err());
    }
}
