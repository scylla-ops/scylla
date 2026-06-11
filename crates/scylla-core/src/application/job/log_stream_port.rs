use crate::domain::entities::{JobId, JobLog};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::pipeline::NodeId;
use async_trait::async_trait;
use futures_core::Stream;
use std::pin::Pin;

/// Live stream of job log entries as they are published on the message broker.
pub type JobLogLiveStream = Pin<Box<dyn Stream<Item = DomainResult<JobLog>> + Send>>;

/// Port for subscribing to live job log events. The historical snapshot is
/// fetched separately via [`JobLogRepository::list_all_by_job`]; this port only
/// exposes the live tail.
#[async_trait]
pub trait JobLogStreamPort: Send + Sync {
    async fn subscribe(
        &self,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<JobLogLiveStream>;
}
