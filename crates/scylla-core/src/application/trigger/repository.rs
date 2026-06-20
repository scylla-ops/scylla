use crate::domain::entities::{PipelineId, Trigger, TriggerId};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence port for [`Trigger`] aggregates. Triggers are listed per pipeline
/// (a pipeline has few of them, so no pagination), and otherwise follow the
/// standard CRUD shape.
#[async_trait]
pub trait TriggerRepository {
    async fn create(&self, trigger: &Trigger) -> DomainResult<Trigger>;

    async fn find_by_id(&self, id: &TriggerId) -> DomainResult<Trigger>;

    async fn update(&self, trigger: &Trigger) -> DomainResult<Trigger>;

    async fn delete(&self, id: &TriggerId) -> DomainResult<()>;

    async fn list_by_pipeline(&self, pipeline_id: &PipelineId) -> DomainResult<Vec<Trigger>>;
}
