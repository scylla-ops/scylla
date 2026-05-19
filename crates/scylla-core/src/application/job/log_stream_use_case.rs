use crate::application::{JobLogLiveStream, JobLogRepository, JobLogStreamPort};
use crate::domain::entities::{JobId, JobLog};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::job::LogStream;
use crate::domain::value_objects::pipeline::NodeId;
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use futures_util::stream::{self, StreamExt, TryStreamExt};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::instrument;

/// Use case that combines a persisted snapshot of job logs with a live broker
/// subscription, exposing them as a single ordered stream.
#[derive(Constructor)]
pub struct JobLogStreamUseCase<R: JobLogRepository, S: JobLogStreamPort> {
    repo: Arc<R>,
    stream_port: Arc<S>,
}

impl<R, S> JobLogStreamUseCase<R, S>
where
    R: JobLogRepository + Send + Sync + 'static,
    S: JobLogStreamPort + 'static,
{
    /// Stream every log of `job_id` (optionally filtered to one node):
    /// 1. Subscribe to the live broker subject first so nothing published while
    ///    we read the snapshot is lost.
    /// 2. Read the historical snapshot from the repository (ordered ASC).
    /// 3. Emit historical first, then forward live messages, de-duplicating
    ///    against the snapshot by `(timestamp, node_id, stream, line)` so that
    ///    same-millisecond / equal-content duplicates aren't dropped or
    ///    double-emitted. Live messages strictly newer than `cutoff` always
    ///    pass; messages at-or-before `cutoff` only pass when their tuple is
    ///    not already in the historical set.
    #[instrument(skip(self), fields(job_id = %job_id, node_id = ?node_id))]
    pub async fn stream(
        &self,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<JobLogLiveStream> {
        let live = self.stream_port.subscribe(job_id, node_id).await?;
        let historical = self.repo.list_all_by_job(job_id, node_id).await?;
        let cutoff = historical.last().map(JobLog::timestamp);

        // Dedup key for the boundary window: any live message whose tuple is
        // already present in the snapshot is a duplicate, regardless of how
        // its timestamp compares to the cutoff.
        let seen: HashSet<(DateTime<Utc>, NodeId, LogStream, String)> = historical
            .iter()
            .map(|l| {
                (
                    l.timestamp(),
                    l.node_id().clone(),
                    *l.stream(),
                    l.line().to_string(),
                )
            })
            .collect();

        let historical_stream = stream::iter(historical.into_iter().map(Ok));
        let filtered_live = live.try_filter(move |log| {
            let strictly_newer = cutoff.is_none_or(|c| log.timestamp() > c);
            let keep = if strictly_newer {
                true
            } else {
                !seen.contains(&(
                    log.timestamp(),
                    log.node_id().clone(),
                    *log.stream(),
                    log.line().to_string(),
                ))
            };
            std::future::ready(keep)
        });

        Ok(Box::pin(historical_stream.chain(filtered_live)))
    }
}
