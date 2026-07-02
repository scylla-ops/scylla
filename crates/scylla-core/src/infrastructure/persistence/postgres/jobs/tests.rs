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

/// The reaper orphans running jobs whose agent is no longer connected, and only
/// those: a running job owned by a connected agent, a pending job, and a
/// terminal job are all left untouched. An empty connected set (boot
/// reconciliation) then orphans every remaining running job.
#[sqlx::test(migrations = "../../migrations")]
async fn orphan_running_without_agents_reaps_only_stranded_running_jobs(pool: PgPool) {
    use crate::application::AppRepository;
    use crate::application::authz::grant::{Grant, ORGANIZATION_AGENT_ROLE, Principal, Scope};
    use crate::domain::entities::{Agent, App, AppCredential};
    use crate::domain::value_objects::app::{AppName, AppSecretHash, AppSecretLabel};
    use crate::domain::value_objects::role::name::RoleName;
    use crate::infrastructure::persistence::postgres::PgAppRepository;

    let (org, _project, pipeline) = seed_org_project_pipeline(&pool, "reap").await;
    let repo = PgJobRepository::new(pool.clone());

    // A live agent to own one of the running jobs.
    let app = App::create(org.id().clone(), AppName::new("live-runner").unwrap());
    let credential = AppCredential::create(
        app.id().clone(),
        AppSecretLabel::new("default").unwrap(),
        AppSecretHash::new("$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$aGFzaGhhc2g").unwrap(),
    );
    let agent = Agent::create(app.id().clone());
    let grant = Grant::new(
        Principal::App(app.id().clone()),
        RoleName::new(ORGANIZATION_AGENT_ROLE).unwrap(),
        Scope::Organization(org.id().clone()),
    );
    PgAppRepository::new(pool.clone())
        .provision_agent(&app, &credential, &agent, &grant)
        .await
        .expect("provision agent");

    // owned: running, assigned to the live (connected) agent.
    let mut owned = job(&pipeline);
    owned.start().unwrap();
    repo.create(&owned).await.unwrap();
    repo.set_agent(owned.id(), app.id()).await.unwrap();
    // stranded: running, no agent.
    let mut stranded = job(&pipeline);
    stranded.start().unwrap();
    repo.create(&stranded).await.unwrap();
    // pending + terminal: never reaped.
    let pending = job(&pipeline);
    repo.create(&pending).await.unwrap();
    let done = JobBuilder::new(&pipeline)
        .terminated(JobStatus::Completed)
        .build();
    repo.create(&done).await.unwrap();

    // With the live agent connected, only the stranded running job is reaped.
    let reaped = repo
        .orphan_running_without_agents(std::slice::from_ref(app.id()))
        .await
        .unwrap();
    assert_eq!(reaped, 1, "only the agent-less running job is orphaned");
    assert_eq!(status(&repo, owned.id()).await, JobStatus::Running);
    assert_eq!(status(&repo, stranded.id()).await, JobStatus::Orphaned);
    assert_eq!(status(&repo, pending.id()).await, JobStatus::Pending);
    assert_eq!(status(&repo, done.id()).await, JobStatus::Completed);
    assert!(
        repo.find_by_id(stranded.id())
            .await
            .unwrap()
            .finished_at()
            .is_some(),
        "an orphaned job is stamped finished",
    );

    // Boot reconciliation: an empty connected set reaps the still-running owned job.
    let reaped_at_boot = repo.orphan_running_without_agents(&[]).await.unwrap();
    assert_eq!(reaped_at_boot, 1, "the last running job is reaped at boot");
    assert_eq!(status(&repo, owned.id()).await, JobStatus::Orphaned);
}

async fn status(repo: &PgJobRepository, id: &crate::domain::entities::JobId) -> JobStatus {
    repo.find_by_id(id).await.unwrap().status()
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
