use crate::application::caller::CallerContext;
use crate::application::{JobLogLiveStream, JobLogRepository, JobLogStreamPort, PermissionService};
use crate::domain::entities::{JobId, JobLog};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::job::LogStream;
use crate::domain::value_objects::permission::policy;
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
pub struct JobLogStreamUseCase<
    R: JobLogRepository,
    S: JobLogStreamPort,
    PS: PermissionService,
> {
    repo: Arc<R>,
    stream_port: Arc<S>,
    permission_service: Arc<PS>,
}

impl<R, S, PS> JobLogStreamUseCase<R, S, PS>
where
    R: JobLogRepository + Send + Sync + 'static,
    S: JobLogStreamPort + 'static,
    PS: PermissionService + 'static,
{
    #[instrument(skip(self, caller), fields(job_id = %job_id, node_id = ?node_id))]
    pub async fn stream(
        &self,
        caller: &CallerContext,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<JobLogLiveStream> {
        self.permission_service
            .check(caller, policy::job::read_logs(job_id.clone()))
            .await?;

        let live = self.stream_port.subscribe(job_id, node_id).await?;
        let historical = self.repo.list_all_by_job(job_id, node_id).await?;
        let cutoff = historical.last().map(JobLog::timestamp);

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
