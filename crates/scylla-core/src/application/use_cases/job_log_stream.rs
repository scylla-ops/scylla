use crate::application::ports::{JobLogLiveStream, JobLogRepository, JobLogStreamPort};
use crate::domain::entities::JobId;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::pipeline::NodeId;
use derive_more::Constructor;
use futures_util::stream::{self, StreamExt, TryStreamExt};
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
    /// 3. Emit historical first, then forward live messages whose timestamp is
    ///    strictly greater than the snapshot's max timestamp (dedup against
    ///    rows the recorder already persisted).
    #[instrument(skip(self), fields(job_id = %job_id, node_id = ?node_id))]
    pub async fn stream(
        &self,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<JobLogLiveStream> {
        let live = self.stream_port.subscribe(job_id, node_id).await?;
        let historical = self.repo.list_all_by_job(job_id, node_id).await?;
        let cutoff = historical.last().map(|log| log.timestamp());

        let historical_stream = stream::iter(historical.into_iter().map(Ok));
        let filtered_live = live.try_filter(move |log| {
            let keep = cutoff.map_or(true, |c| log.timestamp() > c);
            std::future::ready(keep)
        });

        Ok(Box::pin(historical_stream.chain(filtered_live)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::JobLog;
    use crate::domain::value_objects::job::LogStream;
    use async_trait::async_trait;
    use chrono::{DateTime, Duration, Utc};
    use futures_util::StreamExt;
    use std::sync::Mutex;

    fn log_at(ts: DateTime<Utc>, line: &str) -> JobLog {
        JobLog::new(
            JobId::new("job-1"),
            NodeId::new("node-1").unwrap(),
            LogStream::Stdout,
            line.to_string(),
            ts,
        )
    }

    struct FakeRepo {
        logs: Vec<JobLog>,
    }

    #[async_trait]
    impl crate::application::ports::JobLogRepository for FakeRepo {
        async fn create(&self, _: &JobLog) -> DomainResult<JobLog> {
            unimplemented!()
        }
        async fn find_by_id(
            &self,
            _: &crate::domain::entities::JobLogId,
        ) -> DomainResult<JobLog> {
            unimplemented!()
        }
        async fn list_by_job(
            &self,
            _: &JobId,
            _: Option<&crate::domain::value_objects::PaginationParams>,
        ) -> DomainResult<crate::domain::value_objects::PaginatedResult<JobLog>> {
            unimplemented!()
        }
        async fn list_by_job_and_node(
            &self,
            _: &JobId,
            _: &NodeId,
            _: Option<&crate::domain::value_objects::PaginationParams>,
        ) -> DomainResult<crate::domain::value_objects::PaginatedResult<JobLog>> {
            unimplemented!()
        }
        async fn list_all_by_job(
            &self,
            _: &JobId,
            _: Option<&NodeId>,
        ) -> DomainResult<Vec<JobLog>> {
            Ok(self.logs.clone())
        }
    }

    struct FakeStream {
        live: Mutex<Option<Vec<JobLog>>>,
    }

    #[async_trait]
    impl JobLogStreamPort for FakeStream {
        async fn subscribe(
            &self,
            _: &JobId,
            _: Option<&NodeId>,
        ) -> DomainResult<JobLogLiveStream> {
            let logs = self
                .live
                .lock()
                .unwrap()
                .take()
                .unwrap_or_default();
            Ok(Box::pin(stream::iter(logs.into_iter().map(Ok))))
        }
    }

    #[tokio::test]
    async fn stream_emits_historical_then_filtered_live() {
        let t0 = Utc::now();
        let historical = vec![
            log_at(t0, "h1"),
            log_at(t0 + Duration::milliseconds(10), "h2"),
            log_at(t0 + Duration::milliseconds(20), "h3"),
        ];
        let cutoff = t0 + Duration::milliseconds(20);

        let live = vec![
            log_at(t0 + Duration::milliseconds(5), "stale"),
            log_at(cutoff, "boundary"),
            log_at(cutoff + Duration::milliseconds(1), "fresh1"),
            log_at(cutoff + Duration::milliseconds(2), "fresh2"),
        ];

        let repo = Arc::new(FakeRepo { logs: historical });
        let stream_port = Arc::new(FakeStream {
            live: Mutex::new(Some(live)),
        });
        let uc = JobLogStreamUseCase::new(repo, stream_port);

        let mut out = uc
            .stream(&JobId::new("job-1"), None)
            .await
            .expect("stream open");

        let mut collected = Vec::new();
        while let Some(item) = out.next().await {
            collected.push(item.expect("ok"));
        }

        let lines: Vec<&str> = collected.iter().map(|l| l.line()).collect();
        assert_eq!(lines, vec!["h1", "h2", "h3", "fresh1", "fresh2"]);
    }

    #[tokio::test]
    async fn stream_empty_history_passes_all_live() {
        let t0 = Utc::now();
        let live = vec![
            log_at(t0, "l1"),
            log_at(t0 + Duration::milliseconds(1), "l2"),
        ];
        let repo = Arc::new(FakeRepo { logs: vec![] });
        let stream_port = Arc::new(FakeStream {
            live: Mutex::new(Some(live)),
        });
        let uc = JobLogStreamUseCase::new(repo, stream_port);

        let mut out = uc
            .stream(&JobId::new("job-1"), None)
            .await
            .expect("stream open");

        let mut collected = Vec::new();
        while let Some(item) = out.next().await {
            collected.push(item.expect("ok"));
        }
        let lines: Vec<&str> = collected.iter().map(|l| l.line()).collect();
        assert_eq!(lines, vec!["l1", "l2"]);
    }
}
