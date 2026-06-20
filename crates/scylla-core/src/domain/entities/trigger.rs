use crate::domain::clock;
use crate::domain::entities::{PipelineId, TriggerId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::trigger::{
    TriggerInput, TriggerInputSource, TriggerName, TriggerSource,
};
use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// A stored initiator that launches runs of a single pipeline. Firing always
/// flows through the normal `PipelineUseCases::run` path (one `RunPipeline`
/// check, one job minted) — a trigger is a new *source*, not a new execution
/// path. The `source` decides *how* it fires (cron schedule, inbound webhook);
/// `inputs` overlay literal env on each fired run.
#[derive(Debug, Clone)]
pub struct Trigger {
    id: TriggerId,
    pipeline_id: PipelineId,
    name: TriggerName,
    source: TriggerSource,
    inputs: Vec<TriggerInput>,
    enabled: bool,
    /// Cron/poll only: when the next occurrence is due (UTC). `None` for webhook
    /// (push-driven) and until the scheduler computes the first occurrence.
    next_fire_at: Option<DateTime<Utc>>,
    last_fired_at: Option<DateTime<Utc>>,
    /// Outcome of the last fire attempt, for observability (`"ok"` / an error).
    last_status: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Trigger {
    /// Reconstitute a `Trigger` from persistent storage without re-validation;
    /// fields were validated at create/update time and JSONB is round-tripped
    /// verbatim.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn from_persistence(
        id: TriggerId,
        pipeline_id: PipelineId,
        name: TriggerName,
        source: TriggerSource,
        inputs: Vec<TriggerInput>,
        enabled: bool,
        next_fire_at: Option<DateTime<Utc>>,
        last_fired_at: Option<DateTime<Utc>>,
        last_status: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            pipeline_id,
            name,
            source,
            inputs,
            enabled,
            next_fire_at,
            last_fired_at,
            last_status,
            created_at,
            updated_at,
        }
    }

    /// Create a new, enabled trigger. Validates that the inputs are coherent with
    /// the source kind (a JSON-pointer input requires a webhook payload).
    pub fn create(
        pipeline_id: PipelineId,
        name: TriggerName,
        source: TriggerSource,
        inputs: Vec<TriggerInput>,
    ) -> DomainResult<Self> {
        Self::validate_inputs(&source, &inputs)?;

        let now = clock::now();
        Ok(Self {
            id: TriggerId::generate(),
            pipeline_id,
            name,
            source,
            inputs,
            enabled: true,
            next_fire_at: None,
            last_fired_at: None,
            last_status: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update the editable fields. The source *kind* is immutable — switching
    /// cron↔webhook requires delete + recreate (different secret/URL/schedule
    /// lifecycle).
    pub fn update(
        &mut self,
        name: TriggerName,
        source: TriggerSource,
        inputs: Vec<TriggerInput>,
    ) -> DomainResult<()> {
        if source.kind() != self.source.kind() {
            return Err(DomainError::business_rule(
                "Trigger source kind is immutable; delete and recreate to change it",
            ));
        }
        Self::validate_inputs(&source, &inputs)?;
        self.name = name;
        self.source = source;
        self.inputs = inputs;
        self.updated_at = clock::now();
        Ok(())
    }

    /// Enable or disable the trigger. A disabled trigger never fires.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.updated_at = clock::now();
    }

    /// Set the next due time (the scheduler owns this for cron sources).
    pub fn set_next_fire_at(&mut self, next_fire_at: Option<DateTime<Utc>>) {
        self.next_fire_at = next_fire_at;
        self.updated_at = clock::now();
    }

    /// Record the outcome of a fire attempt.
    pub fn mark_fired(&mut self, fired_at: DateTime<Utc>, status: impl Into<String>) {
        self.last_fired_at = Some(fired_at);
        self.last_status = Some(status.into());
        self.updated_at = clock::now();
    }

    fn validate_inputs(source: &TriggerSource, inputs: &[TriggerInput]) -> DomainResult<()> {
        let mut seen = HashSet::new();
        for input in inputs {
            if !seen.insert(input.key()) {
                return Err(DomainError::validation(format!(
                    "Duplicate input key: {}",
                    input.key()
                )));
            }
            if matches!(source, TriggerSource::Cron(_))
                && matches!(input.source(), TriggerInputSource::JsonPointer(_))
            {
                return Err(DomainError::validation(format!(
                    "Input '{}' uses a JSON pointer, which requires a webhook source (cron has no payload)",
                    input.key()
                )));
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn id(&self) -> &TriggerId {
        &self.id
    }

    #[must_use]
    pub fn pipeline_id(&self) -> &PipelineId {
        &self.pipeline_id
    }

    #[must_use]
    pub fn name(&self) -> &TriggerName {
        &self.name
    }

    #[must_use]
    pub fn source(&self) -> &TriggerSource {
        &self.source
    }

    #[must_use]
    pub fn inputs(&self) -> &[TriggerInput] {
        &self.inputs
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub fn next_fire_at(&self) -> Option<DateTime<Utc>> {
        self.next_fire_at
    }

    #[must_use]
    pub fn last_fired_at(&self) -> Option<DateTime<Utc>> {
        self.last_fired_at
    }

    #[must_use]
    pub fn last_status(&self) -> Option<&str> {
        self.last_status.as_deref()
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::pipeline::EnvKey;
    use crate::domain::value_objects::trigger::{CronSpec, WebhookSpec};

    fn pipeline_id() -> PipelineId {
        PipelineId::generate()
    }

    fn name(n: &str) -> TriggerName {
        TriggerName::new(n).unwrap()
    }

    fn cron() -> TriggerSource {
        TriggerSource::Cron(CronSpec::new("0 9 * * *").unwrap())
    }

    fn webhook() -> TriggerSource {
        TriggerSource::Webhook(WebhookSpec::new(None).unwrap())
    }

    fn key(k: &str) -> EnvKey {
        EnvKey::new(k).unwrap()
    }

    #[test]
    fn create_cron_trigger_is_enabled_and_unscheduled() {
        let t = Trigger::create(pipeline_id(), name("nightly"), cron(), vec![]).unwrap();
        assert!(t.is_enabled());
        assert!(t.next_fire_at().is_none());
        assert!(t.last_fired_at().is_none());
    }

    #[test]
    fn cron_rejects_json_pointer_inputs() {
        let inputs = vec![TriggerInput::json_pointer(key("GIT_COMMIT"), "/after").unwrap()];
        let err = Trigger::create(pipeline_id(), name("bad"), cron(), inputs);
        assert!(err.is_err());
    }

    #[test]
    fn webhook_accepts_json_pointer_inputs() {
        let inputs = vec![TriggerInput::json_pointer(key("GIT_COMMIT"), "/after").unwrap()];
        assert!(Trigger::create(pipeline_id(), name("on-push"), webhook(), inputs).is_ok());
    }

    #[test]
    fn cron_accepts_literal_inputs() {
        let inputs = vec![TriggerInput::literal(key("RUN_MODE"), "nightly")];
        assert!(Trigger::create(pipeline_id(), name("nightly"), cron(), inputs).is_ok());
    }

    #[test]
    fn rejects_duplicate_input_keys() {
        let inputs = vec![
            TriggerInput::literal(key("RUN_MODE"), "a"),
            TriggerInput::literal(key("RUN_MODE"), "b"),
        ];
        assert!(Trigger::create(pipeline_id(), name("dup"), cron(), inputs).is_err());
    }

    #[test]
    fn update_cannot_change_kind() {
        let mut t = Trigger::create(pipeline_id(), name("nightly"), cron(), vec![]).unwrap();
        let err = t.update(name("nightly"), webhook(), vec![]);
        assert!(err.is_err());
        assert_eq!(t.source().kind().as_str(), "cron");
    }

    #[test]
    fn update_same_kind_succeeds() {
        let mut t = Trigger::create(pipeline_id(), name("nightly"), cron(), vec![]).unwrap();
        let new_source = TriggerSource::Cron(CronSpec::new("0 0 * * *").unwrap());
        t.update(name("midnight"), new_source, vec![]).unwrap();
        assert_eq!(t.name().as_str(), "midnight");
    }

    #[test]
    fn set_enabled_and_mark_fired_track_state() {
        let mut t = Trigger::create(pipeline_id(), name("nightly"), cron(), vec![]).unwrap();
        t.set_enabled(false);
        assert!(!t.is_enabled());

        let now = clock::now();
        t.mark_fired(now, "ok");
        assert_eq!(t.last_fired_at(), Some(now));
        assert_eq!(t.last_status(), Some("ok"));
    }
}
