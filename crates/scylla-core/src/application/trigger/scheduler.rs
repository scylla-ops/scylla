use crate::application::trigger::fire::TriggerFiring;
use crate::application::trigger::repository::TriggerRepository;
use crate::application::trigger::schedule::{CronSchedule, next_fire_time};
use crate::domain::clock;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::trigger::Trigger;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::{info, instrument, warn};

const CRON_CLAIM_BATCH: i64 = 100;

/// Missed occurrences during downtime are skipped, not backfilled.
pub struct TriggerCronScheduler {
    trigger_repo: Arc<dyn TriggerRepository>,
    firing: Arc<dyn TriggerFiring>,
    schedule: Arc<dyn CronSchedule>,
}

impl TriggerCronScheduler {
    #[must_use]
    pub fn new(
        trigger_repo: Arc<dyn TriggerRepository>,
        firing: Arc<dyn TriggerFiring>,
        schedule: Arc<dyn CronSchedule>,
    ) -> Self {
        Self {
            trigger_repo,
            firing,
            schedule,
        }
    }

    #[instrument(skip(self))]
    pub async fn tick(&self) -> usize {
        self.seed_unscheduled().await;
        self.fire_due().await
    }

    async fn seed_unscheduled(&self) {
        let unscheduled = match self.trigger_repo.list_unscheduled_cron().await {
            Ok(t) => t,
            Err(e) => {
                warn!(error = %e, "cron seed: could not list unscheduled triggers");
                return;
            }
        };
        for mut trigger in unscheduled {
            match next_fire_time(&trigger, &*self.schedule, clock::now()) {
                Ok(Some(next)) => {
                    trigger.set_next_fire_at(Some(next));
                    if let Err(e) = self.trigger_repo.update(&trigger).await {
                        warn!(trigger_id = %trigger.id(), error = %e, "cron seed: could not persist next_fire_at");
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    warn!(trigger_id = %trigger.id(), error = %e, "cron seed: invalid expression; trigger will not fire");
                }
            }
        }
    }

    async fn fire_due(&self) -> usize {
        let now = clock::now();
        // A clone, not `&self`, so the callback is `Sync` whatever `T` is.
        let schedule = self.schedule.clone();
        let compute_next = move |trigger: &Trigger| -> DomainResult<DateTime<Utc>> {
            next_fire_time(trigger, &*schedule, now)?
                .ok_or_else(|| DomainError::internal("non-cron trigger reached the cron claim"))
        };

        let due = match self
            .trigger_repo
            .claim_due_cron(now, CRON_CLAIM_BATCH, &compute_next)
            .await
        {
            Ok(due) => due,
            Err(e) => {
                warn!(error = %e, "cron claim: could not claim due triggers");
                return 0;
            }
        };

        let mut fired = 0usize;
        for trigger in due {
            match self.firing.fire(trigger.id(), None, None).await {
                Ok(job) => {
                    info!(trigger_id = %trigger.id(), job_id = %job.id(), "cron trigger fired");
                    fired += 1;
                }
                // The occurrence is already consumed; one bad trigger must not stall the rest.
                Err(e) => {
                    warn!(trigger_id = %trigger.id(), error = %e, "cron trigger fire failed");
                }
            }
        }
        if fired > 0 {
            info!(fired, "cron scheduler fired due triggers");
        }
        fired
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::{PipelineId, TriggerId};
    use crate::domain::job::Job;
    use crate::domain::trigger::Trigger;
    use crate::domain::trigger::{CronSpec, TriggerName, TriggerSource, WebhookSpec};
    use async_trait::async_trait;
    use std::sync::Mutex;

    fn cron_trigger(name: &str) -> Trigger {
        Trigger::create(
            PipelineId::new("p"),
            TriggerName::new(name).unwrap(),
            TriggerSource::Cron(CronSpec::new("* * * * *").unwrap()),
            vec![],
        )
        .unwrap()
    }

    struct StubRepo {
        unscheduled: Vec<Trigger>,
        due: Vec<Trigger>,
        updated: Mutex<Vec<(String, Option<DateTime<Utc>>)>>,
    }

    #[async_trait]
    impl TriggerRepository for StubRepo {
        async fn list_unscheduled_cron(&self) -> DomainResult<Vec<Trigger>> {
            Ok(self.unscheduled.clone())
        }
        async fn claim_due_cron(
            &self,
            _now: DateTime<Utc>,
            _limit: i64,
            compute_next: &(dyn for<'a> Fn(&'a Trigger) -> DomainResult<DateTime<Utc>> + Sync),
        ) -> DomainResult<Vec<Trigger>> {
            for t in &self.due {
                let _ = compute_next(t);
            }
            Ok(self.due.clone())
        }
        async fn update(&self, trigger: &Trigger) -> DomainResult<Trigger> {
            self.updated
                .lock()
                .unwrap()
                .push((trigger.id().to_string(), trigger.next_fire_at()));
            Ok(trigger.clone())
        }
        async fn create(&self, _: &Trigger, _: Option<&[u8]>) -> DomainResult<Trigger> {
            unimplemented!()
        }
        async fn find_by_id(&self, _: &TriggerId) -> DomainResult<Trigger> {
            unimplemented!()
        }
        async fn webhook_secret(&self, _: &TriggerId) -> DomainResult<Option<Vec<u8>>> {
            unimplemented!()
        }
        async fn delete(&self, _: &TriggerId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_by_pipeline(&self, _: &PipelineId) -> DomainResult<Vec<Trigger>> {
            unimplemented!()
        }
    }

    struct StubFiring {
        job: Job,
        fired: Mutex<Vec<String>>,
        fail: Vec<String>,
    }

    #[async_trait]
    impl TriggerFiring for StubFiring {
        async fn fire(
            &self,
            trigger_id: &TriggerId,
            _payload: Option<&serde_json::Value>,
            _delivery_id: Option<&str>,
        ) -> DomainResult<Job> {
            self.fired.lock().unwrap().push(trigger_id.to_string());
            if self.fail.contains(&trigger_id.to_string()) {
                return Err(DomainError::internal("boom"));
            }
            Ok(self.job.clone())
        }
    }

    struct StubSchedule;

    impl CronSchedule for StubSchedule {
        fn next_after(
            &self,
            expression: &str,
            after: DateTime<Utc>,
        ) -> DomainResult<DateTime<Utc>> {
            if expression == "bad" {
                return Err(DomainError::validation("bad expression"));
            }
            Ok(after + chrono::Duration::seconds(60))
        }
    }

    fn a_job() -> Job {
        use crate::test_support::{
            jobs::job, organizations::org, pipelines::pipeline, projects::project,
        };
        job(&pipeline(&project(&org("o"), "p")))
    }

    #[tokio::test]
    async fn tick_fires_all_due_triggers() {
        let due = vec![cron_trigger("a"), cron_trigger("b")];
        let repo = Arc::new(StubRepo {
            unscheduled: vec![],
            due: due.clone(),
            updated: Mutex::new(vec![]),
        });
        let firing = Arc::new(StubFiring {
            job: a_job(),
            fired: Mutex::new(vec![]),
            fail: vec![],
        });
        let scheduler = TriggerCronScheduler::new(repo, firing.clone(), Arc::new(StubSchedule));

        assert_eq!(scheduler.tick().await, 2);
        assert_eq!(firing.fired.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn one_fire_failure_does_not_abort_the_pass() {
        let a = cron_trigger("a");
        let b = cron_trigger("b");
        let fail_id = a.id().to_string();
        let repo = Arc::new(StubRepo {
            unscheduled: vec![],
            due: vec![a, b],
            updated: Mutex::new(vec![]),
        });
        let firing = Arc::new(StubFiring {
            job: a_job(),
            fired: Mutex::new(vec![]),
            fail: vec![fail_id],
        });
        let scheduler = TriggerCronScheduler::new(repo, firing.clone(), Arc::new(StubSchedule));

        assert_eq!(scheduler.tick().await, 1);
        assert_eq!(firing.fired.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn seeding_sets_next_fire_at_and_persists() {
        let repo = Arc::new(StubRepo {
            unscheduled: vec![cron_trigger("fresh")],
            due: vec![],
            updated: Mutex::new(vec![]),
        });
        let firing = Arc::new(StubFiring {
            job: a_job(),
            fired: Mutex::new(vec![]),
            fail: vec![],
        });
        let scheduler = TriggerCronScheduler::new(repo.clone(), firing, Arc::new(StubSchedule));

        scheduler.tick().await;
        let updated = repo.updated.lock().unwrap();
        assert_eq!(updated.len(), 1);
        assert!(updated[0].1.is_some(), "next_fire_at was seeded");
    }

    #[tokio::test]
    async fn webhook_trigger_is_never_seeded_as_cron() {
        let webhook = Trigger::create(
            PipelineId::new("p"),
            TriggerName::new("hook").unwrap(),
            TriggerSource::Webhook(WebhookSpec::new(None).unwrap()),
            vec![],
        )
        .unwrap();
        let repo = Arc::new(StubRepo {
            unscheduled: vec![webhook],
            due: vec![],
            updated: Mutex::new(vec![]),
        });
        let firing = Arc::new(StubFiring {
            job: a_job(),
            fired: Mutex::new(vec![]),
            fail: vec![],
        });
        let scheduler = TriggerCronScheduler::new(repo.clone(), firing, Arc::new(StubSchedule));

        scheduler.tick().await;
        assert!(
            repo.updated.lock().unwrap().is_empty(),
            "no update for webhook"
        );
    }
}
