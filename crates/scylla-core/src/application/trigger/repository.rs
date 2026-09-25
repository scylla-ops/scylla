use crate::domain::errors::DomainResult;
use crate::domain::ids::{PipelineId, TriggerId};
use crate::domain::trigger::Trigger;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait TriggerRepository: Send + Sync {
    async fn create(
        &self,
        trigger: &Trigger,
        webhook_secret_enc: Option<&[u8]>,
    ) -> DomainResult<Trigger>;

    async fn find_by_id(&self, id: &TriggerId) -> DomainResult<Trigger>;

    async fn webhook_secret(&self, id: &TriggerId) -> DomainResult<Option<Vec<u8>>>;

    async fn update(&self, trigger: &Trigger) -> DomainResult<Trigger>;

    async fn delete(&self, id: &TriggerId) -> DomainResult<()>;

    async fn list_by_pipeline(&self, pipeline_id: &PipelineId) -> DomainResult<Vec<Trigger>>;

    async fn list_unscheduled_cron(&self) -> DomainResult<Vec<Trigger>>;

    /// `FOR UPDATE SKIP LOCKED` and the advance in the same transaction: no pass or instance re-claims a row.
    async fn claim_due_cron(
        &self,
        now: DateTime<Utc>,
        limit: i64,
        compute_next: &(dyn for<'a> Fn(&'a Trigger) -> DomainResult<DateTime<Utc>> + Sync),
    ) -> DomainResult<Vec<Trigger>>;
}
