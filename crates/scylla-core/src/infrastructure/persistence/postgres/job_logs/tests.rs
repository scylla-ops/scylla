use super::PgJobLogRepository;
use crate::application::ports::{JobLogRepository, JobRepository};
use crate::domain::entities::JobLog;
use crate::infrastructure::persistence::postgres::PgJobRepository;
use crate::test_support::prelude::*;
use chrono::{Duration, Utc};
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn create_then_find_by_id(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "l").await;
    let job = seed_job(&pool, &pipeline).await;
    let repo = PgJobLogRepository::new(pool);

    let log = job_log(job.id(), "a", "hello");
    repo.create(&log).await.unwrap();

    let found = repo.find_by_id(log.id()).await.unwrap();
    assert_eq!(found.line(), "hello");
    assert_eq!(found.node_id().as_str(), "a");
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_by_job_orders_by_timestamp_ascending(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "ord").await;
    let job = seed_job(&pool, &pipeline).await;
    let repo = PgJobLogRepository::new(pool);

    // Insert in reverse temporal order to prove the query sorts, not the insert order.
    let base = Utc::now();
    for (line, offset_secs) in [("third", 0_i64), ("second", -1), ("first", -2)] {
        let log = JobLogBuilder::new(job.id(), "a", line)
            .timestamp(base + Duration::seconds(offset_secs))
            .build();
        repo.create(&log).await.unwrap();
    }

    let listed = repo.list_all_by_job(job.id(), None).await.unwrap();
    let lines: Vec<&str> = listed.iter().map(JobLog::line).collect();
    assert_eq!(lines, vec!["first", "second", "third"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_by_job_and_node_filters_other_nodes(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "f").await;
    let job = seed_job(&pool, &pipeline).await;
    let repo = PgJobLogRepository::new(pool);

    repo.create(&job_log(job.id(), "a", "log-a-1")).await.unwrap();
    repo.create(&job_log(job.id(), "a", "log-a-2")).await.unwrap();
    repo.create(&job_log(job.id(), "b", "log-b-1")).await.unwrap();

    let target = crate::domain::value_objects::pipeline::NodeId::new("a").unwrap();
    let scoped = repo.list_all_by_job(job.id(), Some(&target)).await.unwrap();
    assert_eq!(scoped.len(), 2);
    assert!(scoped.iter().all(|l| l.node_id().as_str() == "a"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_all_by_job_returns_full_history(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "h").await;
    let job = seed_job(&pool, &pipeline).await;
    let repo = PgJobLogRepository::new(pool);

    for i in 0..5 {
        repo.create(&job_log(job.id(), "a", &format!("line-{i}"))).await.unwrap();
    }

    assert_eq!(repo.list_all_by_job(job.id(), None).await.unwrap().len(), 5);
}

#[sqlx::test(migrations = "../../migrations")]
async fn cascade_job_delete_removes_logs(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "c").await;
    let job = seed_job(&pool, &pipeline).await;
    let log_repo = PgJobLogRepository::new(pool.clone());
    log_repo.create(&job_log(job.id(), "a", "doomed")).await.unwrap();

    PgJobRepository::new(pool).delete(job.id()).await.unwrap();

    assert!(log_repo.list_all_by_job(job.id(), None).await.unwrap().is_empty());
}
