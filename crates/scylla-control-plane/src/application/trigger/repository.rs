use crate::domain::errors::DomainResult;
use crate::domain::ids::{PipelineId, TriggerId};
use crate::domain::trigger::Trigger;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Persistence port for [`Trigger`] aggregates. Triggers are listed per pipeline
/// (a pipeline has few of them, so no pagination), and otherwise follow the
/// standard CRUD shape. The cron scheduler additionally needs the two
/// global, kind-scoped reads at the bottom.
#[async_trait]
pub trait TriggerRepository {
    /// Persist a new trigger. `webhook_secret_enc` is the AEAD-encrypted HMAC
    /// signing secret for webhook triggers (written to a column that normal reads
    /// never select); `None` for cron.
    async fn create(
        &self,
        trigger: &Trigger,
        webhook_secret_enc: Option<&[u8]>,
    ) -> DomainResult<Trigger>;

    async fn find_by_id(&self, id: &TriggerId) -> DomainResult<Trigger>;

    /// The AEAD-encrypted webhook signing secret for `id`, or `None` when the
    /// trigger has none (cron, or never set). Read only on the ingress path.
    async fn webhook_secret(&self, id: &TriggerId) -> DomainResult<Option<Vec<u8>>>;

    async fn update(&self, trigger: &Trigger) -> DomainResult<Trigger>;

    async fn delete(&self, id: &TriggerId) -> DomainResult<()>;

    async fn list_by_pipeline(&self, pipeline_id: &PipelineId) -> DomainResult<Vec<Trigger>>;

    /// Enabled cron triggers that have no computed `next_fire_at` yet (freshly
    /// created or re-enabled). The scheduler seeds their first occurrence.
    async fn list_unscheduled_cron(&self) -> DomainResult<Vec<Trigger>>;

    /// Atomically claim up to `limit` enabled cron triggers whose `next_fire_at`
    /// is due (`<= now`). Each claimed row is locked `FOR UPDATE SKIP LOCKED` and
    /// its `next_fire_at` is advanced — in the *same* transaction — to
    /// `compute_next(trigger)` (the next occurrence after `now`), so a concurrent
    /// scheduler pass or instance never re-claims it. Returns the claimed triggers
    /// as they were *before* the advance (their due time), for firing. A trigger
    /// whose `compute_next` fails is left untouched and excluded.
    async fn claim_due_cron(
        &self,
        now: DateTime<Utc>,
        limit: i64,
        compute_next: &(dyn for<'a> Fn(&'a Trigger) -> DomainResult<DateTime<Utc>> + Sync),
    ) -> DomainResult<Vec<Trigger>>;
}
