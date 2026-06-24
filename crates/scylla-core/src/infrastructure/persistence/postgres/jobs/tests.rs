use super::PgJobRepository;
use crate::application::{JobRepository, PipelineRepository};
use crate::domain::entities::TriggerId;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::job::{JobOrigin, JobStatus};
use crate::infrastructure::persistence::postgres::PgPipelineRepository;
use crate::test_support::prelude::*;
use chrono::Utc;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn create_then_find_round_trips_node_executions_jsonb(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "rt").await;
    let repo = PgJobRepository::new(pool);
    let job = job(&pipeline);
    repo.create(&job).await.unwrap();

    let found = repo.find_by_id(job.id()).await.unwrap();
    assert_eq!(found.node_executions().len(), pipeline.nodes().len());
    assert_eq!(found.status(), JobStatus::Pending);
    assert!(found.started_at().is_none());
    assert!(found.finished_at().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn status_all_terminal_variants_round_trip(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "st").await;
    let repo = PgJobRepository::new(pool);

    for terminal in [
        JobStatus::Completed,
        JobStatus::Failed,
        JobStatus::Cancelled,
        JobStatus::Orphaned,
    ] {
        let job = JobBuilder::new(&pipeline).terminated(terminal).build();
        repo.create(&job).await.unwrap();
        assert_eq!(repo.find_by_id(job.id()).await.unwrap().status(), terminal);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_persists_started_finished_timestamps(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "ts").await;
    let repo = PgJobRepository::new(pool);
    let mut job = job(&pipeline);
    repo.create(&job).await.unwrap();

    job.start().unwrap();
    job.complete().unwrap();
    repo.update(&job).await.unwrap();

    let found = repo.find_by_id(job.id()).await.unwrap();
    assert!(found.started_at().is_some());
    assert!(found.finished_at().is_some());
    assert!(found.finished_at().unwrap() >= found.started_at().unwrap());
    assert!(found.updated_at() <= Utc::now());
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_by_pipeline_filters(pool: PgPool) {
    let org = seed_org(&pool, "acme").await;
    let project = seed_project(&pool, &org, "p").await;
    let pipe_repo = PgPipelineRepository::new(pool.clone());
    let pipeline_a = pipeline(&project);
    let pipeline_b = pipeline(&project);
    pipe_repo.create(&pipeline_a).await.unwrap();
    pipe_repo.create(&pipeline_b).await.unwrap();

    let repo = PgJobRepository::new(pool);
    repo.create(&job(&pipeline_a)).await.unwrap();
    repo.create(&job(&pipeline_a)).await.unwrap();
    repo.create(&job(&pipeline_b)).await.unwrap();

    assert_eq!(
        repo.list_by_pipeline(pipeline_a.id(), None)
            .await
            .unwrap()
            .metadata()
            .total_count(),
        2,
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_by_organization_joins_through_pipeline_and_project(pool: PgPool) {
    let (org_target, _, pipeline_target) = seed_org_project_pipeline(&pool, "t").await;
    let (_, _, pipeline_other) = seed_org_project_pipeline(&pool, "o").await;
    let repo = PgJobRepository::new(pool);

    repo.create(&job(&pipeline_target)).await.unwrap();
    repo.create(&job(&pipeline_target)).await.unwrap();
    repo.create(&job(&pipeline_other)).await.unwrap();

    assert_eq!(
        repo.list_by_organization(org_target.id(), None)
            .await
            .unwrap()
            .metadata()
            .total_count(),
        2,
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn origin_round_trips_through_jsonb(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "or").await;
    let repo = PgJobRepository::new(pool);

    // Webhook is the richest variant: a typed trigger id plus an optional delivery
    // id — exercises the full JSONB serde round-trip of the tagged union.
    let origin = JobOrigin::Webhook {
        trigger_id: TriggerId::new("trg-1"),
        delivery_id: Some("gh-42".to_string()),
    };
    let job = JobBuilder::new(&pipeline).origin(origin.clone()).build();
    repo.create(&job).await.unwrap();

    let found = repo.find_by_id(job.id()).await.unwrap();
    assert_eq!(found.origin(), &origin);
}

#[sqlx::test(migrations = "../../migrations")]
async fn cascade_pipeline_delete_removes_jobs(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "c").await;
    let repo = PgJobRepository::new(pool.clone());
    let job = job(&pipeline);
    repo.create(&job).await.unwrap();

    PgPipelineRepository::new(pool)
        .delete(pipeline.id())
        .await
        .unwrap();

    assert!(matches!(
        repo.find_by_id(job.id()).await,
        Err(DomainError::NotFound { .. }),
    ));
}
