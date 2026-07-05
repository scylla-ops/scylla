use crate::domain::clock;
use crate::domain::entities::{PipelineId, TriggerId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::trigger::{
    TriggerInput, TriggerInputSource, TriggerName, TriggerSource,
};
use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// Whether a trigger fires, and — for an enabled cron — when it is next due.
/// A disabled trigger has no schedule, so `next_fire_at` cannot outlive being
/// disabled: it lives only inside the `Enabled` variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerActivation {
    Disabled,
    /// Fires. `next_fire_at` is the next due occurrence (cron only, once the
    /// scheduler has computed it); `None` for a webhook or a not-yet-seeded cron.
    Enabled { next_fire_at: Option<DateTime<Utc>> },
}

/// The outcome of the most recent fire attempt. The timestamp and its status
/// move together — there is never one without the other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireObservation {
    pub fired_at: DateTime<Utc>,
    /// `"ok"` or an error description, for observability.
    pub status: String,
}

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
    activation: TriggerActivation,
    last_observation: Option<FireObservation>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Trigger {
    /// Reconstitute a `Trigger` from persistent storage. The flat columns are
    /// normalised into the state machine here — a disabled trigger drops any stale
    /// `next_fire_at`, and a lone `last_status`/`last_fired_at` collapses to a
    /// coherent observation (or none).
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
        let activation = if enabled {
            TriggerActivation::Enabled { next_fire_at }
        } else {
            TriggerActivation::Disabled
        };
        let last_observation = last_fired_at.map(|fired_at| FireObservation {
            fired_at,
            status: last_status.unwrap_or_default(),
        });
        Self {
            id,
            pipeline_id,
            name,
            source,
            inputs,
            activation,
            last_observation,
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
            activation: TriggerActivation::Enabled { next_fire_at: None },
            last_observation: None,
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

    /// Enable the trigger. Its schedule is re-anchored separately (the use case
    /// computes the next occurrence right after), so it starts unscheduled.
    pub fn enable(&mut self) {
        self.activation = TriggerActivation::Enabled { next_fire_at: None };
        self.updated_at = clock::now();
    }

    /// Disable the trigger. A disabled trigger never fires and structurally has no
    /// due time — the schedule is dropped with the `Enabled` state.
    pub fn disable(&mut self) {
        self.activation = TriggerActivation::Disabled;
        self.updated_at = clock::now();
    }

    /// Set the next due time (the scheduler owns this for cron sources). A no-op on
    /// a disabled trigger, which by construction has no schedule.
    pub fn set_next_fire_at(&mut self, next_fire_at: Option<DateTime<Utc>>) {
        if let TriggerActivation::Enabled { next_fire_at: slot } = &mut self.activation {
            *slot = next_fire_at;
            self.updated_at = clock::now();
        }
    }

    /// Record the outcome of a fire attempt (timestamp and status together).
    pub fn mark_fired(&mut self, fired_at: DateTime<Utc>, status: impl Into<String>) {
        self.last_observation = Some(FireObservation {
            fired_at,
            status: status.into(),
        });
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
    pub fn activation(&self) -> &TriggerActivation {
        &self.activation
    }

    #[must_use]
    pub fn last_observation(&self) -> Option<&FireObservation> {
        self.last_observation.as_ref()
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        matches!(self.activation, TriggerActivation::Enabled { .. })
    }

    #[must_use]
    pub fn next_fire_at(&self) -> Option<DateTime<Utc>> {
        match &self.activation {
            TriggerActivation::Enabled { next_fire_at } => *next_fire_at,
            TriggerActivation::Disabled => None,
        }
    }

    #[must_use]
    pub fn last_fired_at(&self) -> Option<DateTime<Utc>> {
        self.last_observation.as_ref().map(|o| o.fired_at)
    }

    #[must_use]
    pub fn last_status(&self) -> Option<&str> {
        self.last_observation.as_ref().map(|o| o.status.as_str())
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
    fn disable_drops_the_schedule() {
        let mut t = Trigger::create(pipeline_id(), name("nightly"), cron(), vec![]).unwrap();
        t.set_next_fire_at(Some(clock::now()));
        assert!(t.next_fire_at().is_some());

        t.disable();
        assert!(!t.is_enabled());
        // A disabled trigger structurally cannot carry a due time.
        assert!(t.next_fire_at().is_none());

        // Scheduling a disabled trigger is a no-op.
        t.set_next_fire_at(Some(clock::now()));
        assert!(t.next_fire_at().is_none());
    }

    #[test]
    fn mark_fired_tracks_the_observation_as_a_pair() {
        let mut t = Trigger::create(pipeline_id(), name("nightly"), cron(), vec![]).unwrap();
        let now = clock::now();
        t.mark_fired(now, "ok");
        assert_eq!(t.last_fired_at(), Some(now));
        assert_eq!(t.last_status(), Some("ok"));
    }
}
