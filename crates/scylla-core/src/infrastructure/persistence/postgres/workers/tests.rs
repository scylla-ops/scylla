use super::PgWorkerRepository;
use crate::application::permission::grant::{
    Grant, GrantPrincipal, GrantRepository, GrantScope, WORKER_ROLE,
};
use crate::application::{AppRepository, JobRepository, WorkerRepository};
use crate::domain::clock;
use crate::domain::entities::{App, Job, JobId, OrganizationId, Worker};
use crate::domain::value_objects::app::{AppName, AppSecretHash};
use crate::domain::value_objects::job::JobStatus;
use crate::domain::value_objects::role::name::RoleName;
use crate::infrastructure::persistence::postgres::{
    PgAppRepository, PgGrantRepository, PgJobRepository,
};
use crate::test_support::prelude::*;
use sqlx::PgPool;

const TEST_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$aGFzaGhhc2g";

fn make_worker(org_id: &OrganizationId, name: &str) -> (App, Worker, Grant) {
    let app = App::create(
        org_id.clone(),
        AppName::new(name).unwrap(),
        AppSecretHash::new(TEST_HASH).unwrap(),
    );
    let worker = Worker::create(app.id().clone());
    let grant = Grant::new(
        GrantPrincipal::App(app.id().clone()),
        RoleName::new(WORKER_ROLE).unwrap(),
        GrantScope::Organization(org_id.clone()),
    );
    (app, worker, grant)
}

#[sqlx::test(migrations = "../../migrations")]
async fn provision_worker_persists_app_row_and_grant(pool: PgPool) {
    let org = seed_org(&pool, "Acme").await;
    let app_repo = PgAppRepository::new(pool.clone());
    let worker_repo = PgWorkerRepository::new(pool.clone());
    let (app, worker, grant) = make_worker(org.id(), "ci-runner");

    app_repo
        .provision_worker(&app, &worker, &grant)
        .await
        .expect("provision_worker");

    let found = worker_repo.find_by_app_id(app.id()).await.expect("worker row");
    assert_eq!(found.app_id(), app.id());
    assert!(found.last_seen().is_none(), "fresh worker never seen");

    let list = worker_repo.list_by_organization(org.id()).await.unwrap();
    assert_eq!(list.len(), 1);

    let grants = PgGrantRepository::new(pool.clone()).list_all().await.unwrap();
    assert!(
        grants
            .iter()
            .any(|g| g.principal == GrantPrincipal::App(app.id().clone())),
        "worker grant minted with the app"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn plain_app_is_not_a_worker(pool: PgPool) {
    let org = seed_org(&pool, "Acme").await;
    let app_repo = PgAppRepository::new(pool.clone());
    let worker_repo = PgWorkerRepository::new(pool.clone());

    let app = App::create(
        org.id().clone(),
        AppName::new("bot").unwrap(),
        AppSecretHash::new(TEST_HASH).unwrap(),
    );
    app_repo.create_app(&app).await.expect("create_app");

    assert!(
        worker_repo.find_by_app_id(app.id()).await.is_err(),
        "a plain app has no workers row"
    );
    assert!(worker_repo.list_by_organization(org.id()).await.unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn touch_last_seen_upserts_and_self_heals(pool: PgPool) {
    let org = seed_org(&pool, "Acme").await;
    let app_repo = PgAppRepository::new(pool.clone());
    let worker_repo = PgWorkerRepository::new(pool.clone());

    // App exists but has no workers row yet (e.g. legacy / pre-migration worker).
    let app = App::create(
        org.id().clone(),
        AppName::new("legacy").unwrap(),
        AppSecretHash::new(TEST_HASH).unwrap(),
    );
    app_repo.create_app(&app).await.unwrap();

    let t1 = clock::now();
    worker_repo.touch_last_seen(app.id(), t1).await.expect("self-heal insert");
    let after_first = worker_repo.find_by_app_id(app.id()).await.unwrap();
    assert!(after_first.last_seen().is_some());

    let t2 = t1 + chrono::Duration::seconds(30);
    worker_repo.touch_last_seen(app.id(), t2).await.expect("upsert update");
    let after_second = worker_repo.find_by_app_id(app.id()).await.unwrap();
    assert_eq!(after_second.last_seen().unwrap().timestamp(), t2.timestamp());
}

#[sqlx::test(migrations = "../../migrations")]
async fn worker_stats_aggregate_jobs_by_status(pool: PgPool) {
    let (org, _project, pipeline) = seed_org_project_pipeline(&pool, "stats").await;
    let app_repo = PgAppRepository::new(pool.clone());
    let worker_repo = PgWorkerRepository::new(pool.clone());
    let job_repo = PgJobRepository::new(pool.clone());
    let (app, worker, grant) = make_worker(org.id(), "runner");
    app_repo.provision_worker(&app, &worker, &grant).await.unwrap();

    let mk = |status: JobStatus| {
        let now = clock::now();
        Job::from_persistence(
            JobId::generate(),
            pipeline.id().clone(),
            status,
            vec![],
            Some(app.id().clone()),
            now,
            now,
            None,
            None,
        )
    };
    for status in [
        JobStatus::Completed,
        JobStatus::Completed,
        JobStatus::Failed,
        JobStatus::Running,
    ] {
        job_repo.create(&mk(status)).await.unwrap();
    }

    let stats = worker_repo.worker_stats(app.id()).await.unwrap();
    assert_eq!(stats.total, 4);
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.failed, 1);
    assert_eq!(stats.running, 1);
    assert!(stats.last_run_at.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_worker_keeps_jobs_and_nulls_attribution(pool: PgPool) {
    let (org, _project, pipeline) = seed_org_project_pipeline(&pool, "del").await;
    let app_repo = PgAppRepository::new(pool.clone());
    let worker_repo = PgWorkerRepository::new(pool.clone());
    let job_repo = PgJobRepository::new(pool.clone());
    let (app, worker, grant) = make_worker(org.id(), "runner");
    app_repo.provision_worker(&app, &worker, &grant).await.unwrap();

    let mut job = Job::create_from_pipeline(&pipeline);
    job.assign_worker(app.id().clone());
    let job = job_repo.create(&job).await.unwrap();

    // Deleting the app cascades the workers row; the job survives with NULL.
    app_repo.delete(app.id()).await.unwrap();

    assert!(worker_repo.find_by_app_id(app.id()).await.is_err());
    let reloaded = job_repo.find_by_id(job.id()).await.expect("job survives");
    assert!(
        reloaded.worker_app_id().is_none(),
        "attribution nulled on worker delete"
    );
}
