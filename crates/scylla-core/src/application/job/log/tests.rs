//! The job log's actions through the engine, on stub ports.

use super::*;
use crate::application::PermissionAuthorizer;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, JobId, JobLogId};
use crate::domain::job::JobLog;
use crate::domain::permission::Permission;
use crate::domain::pipeline::NodeId;
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService};
use crate::test_support::job_logs::{JobLogBuilder, job_log};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use futures_util::{StreamExt, stream};
use scylla_auth::authz::PermissionService;
use scylla_extension::{Actions, Hooks};
use std::sync::Mutex;

#[derive(Default)]
struct StubLogs {
    rows: Mutex<Vec<JobLog>>,
    reads: Mutex<Vec<&'static str>>,
}

fn page(items: Vec<JobLog>) -> PaginatedResult<JobLog> {
    let total = items.len() as u64;
    PaginatedResult::new(items, &PaginationParams::default(), total)
}

#[async_trait]
impl JobLogRepository for StubLogs {
    async fn create(&self, log: &JobLog) -> DomainResult<JobLog> {
        self.rows.lock().unwrap().push(log.clone());
        Ok(log.clone())
    }
    async fn find_by_id(&self, _: &JobLogId) -> DomainResult<JobLog> {
        unreachable!("no job log action reads one line by id")
    }
    async fn list_by_job(
        &self,
        _: &JobId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>> {
        self.reads.lock().unwrap().push("job");
        Ok(page(self.rows.lock().unwrap().clone()))
    }
    async fn list_by_job_and_node(
        &self,
        _: &JobId,
        node_id: &NodeId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>> {
        self.reads.lock().unwrap().push("node");
        let rows = self.rows.lock().unwrap();
        Ok(page(
            rows.iter()
                .filter(|l| l.node_id() == node_id)
                .cloned()
                .collect(),
        ))
    }
    async fn list_all_by_job(&self, _: &JobId, _: Option<&NodeId>) -> DomainResult<Vec<JobLog>> {
        Ok(self.rows.lock().unwrap().clone())
    }
}

#[derive(Default)]
struct StubLive {
    lines: Mutex<Vec<JobLog>>,
    subscribes: Mutex<usize>,
}

#[async_trait]
impl JobLogStreamPort for StubLive {
    async fn subscribe(&self, _: &JobId, _: Option<&NodeId>) -> DomainResult<JobLogLiveStream> {
        *self.subscribes.lock().unwrap() += 1;
        let lines = self.lines.lock().unwrap().clone();
        Ok(Box::pin(stream::iter(lines.into_iter().map(Ok))))
    }
}

struct Lab {
    actions: Actions,
    uc: JobLogUseCases,
    logs: Arc<StubLogs>,
    live: Arc<StubLive>,
}

fn lab(permissions: Arc<dyn PermissionService>) -> Lab {
    let logs = Arc::new(StubLogs::default());
    let live = Arc::new(StubLive::default());
    Lab {
        actions: Actions::new(
            Arc::new(PermissionAuthorizer::new(permissions)),
            Arc::new(Hooks::new()),
        ),
        uc: JobLogUseCases::new(logs.clone(), live.clone()),
        logs,
        live,
    }
}

fn agent() -> CallerContext {
    CallerContext::App(AppId::new("agent"))
}

#[tokio::test]
async fn an_append_checks_append_job_log_and_stores_the_line() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let job_id = JobId::new("job-1");
    let log = job_log(&job_id, "build", "hello");

    let stored = lab
        .actions
        .run(&lab.uc, &agent(), AppendJobLog { log: log.clone() })
        .await
        .unwrap();

    assert_eq!(stored.id(), log.id());
    assert_eq!(lab.logs.rows.lock().unwrap().len(), 1);
    assert_eq!(
        permissions.permissions(),
        vec![Permission::AppendJobLog(job_id)]
    );
}

#[tokio::test]
async fn a_denied_append_stores_nothing() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));
    let log = job_log(&JobId::new("job-1"), "build", "hello");

    let err = lab
        .actions
        .run(&lab.uc, &agent(), AppendJobLog { log })
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.logs.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_list_reads_every_node_or_the_one_it_names() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let job_id = JobId::new("job-1");
    lab.logs.rows.lock().unwrap().extend([
        job_log(&job_id, "build", "a"),
        job_log(&job_id, "test", "b"),
    ]);

    let all = lab
        .actions
        .run(
            &lab.uc,
            &agent(),
            ListJobLogs {
                job_id: job_id.clone(),
                node_id: None,
                pagination: None,
            },
        )
        .await
        .unwrap();
    let one = lab
        .actions
        .run(
            &lab.uc,
            &agent(),
            ListJobLogs {
                job_id: job_id.clone(),
                node_id: Some(NodeId::new("test").unwrap()),
                pagination: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(all.items().len(), 2);
    assert_eq!(one.items().len(), 1);
    assert_eq!(*lab.logs.reads.lock().unwrap(), vec!["job", "node"]);
    assert_eq!(
        permissions.permissions(),
        vec![
            Permission::ReadJobLogs(job_id.clone()),
            Permission::ReadJobLogs(job_id),
        ]
    );
}

#[tokio::test]
async fn a_tail_replays_the_snapshot_then_only_unseen_live_lines() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let job_id = JobId::new("job-1");
    let at = Utc::now();
    let stored = JobLogBuilder::new(&job_id, "build", "old")
        .timestamp(at)
        .build();
    let newer = JobLogBuilder::new(&job_id, "build", "new")
        .timestamp(at + Duration::seconds(1))
        .build();
    lab.logs.rows.lock().unwrap().push(stored.clone());
    lab.live
        .lines
        .lock()
        .unwrap()
        .extend([stored.clone(), newer.clone()]);

    let tail = lab
        .actions
        .run(
            &lab.uc,
            &agent(),
            TailJobLogs {
                job_id: job_id.clone(),
                node_id: None,
            },
        )
        .await
        .unwrap();
    let lines: Vec<String> = tail.map(|l| l.unwrap().line().to_string()).collect().await;

    assert_eq!(lines, vec!["old", "new"]);
    assert_eq!(
        permissions.permissions(),
        vec![Permission::ReadJobLogs(job_id)]
    );
}

#[tokio::test]
async fn a_denied_tail_never_subscribes() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));

    let err = lab
        .actions
        .run(
            &lab.uc,
            &agent(),
            TailJobLogs {
                job_id: JobId::new("job-1"),
                node_id: None,
            },
        )
        .await
        .err()
        .unwrap();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert_eq!(*lab.live.subscribes.lock().unwrap(), 0);
}
