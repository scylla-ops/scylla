use super::PgAgentRepository;
use crate::application::authz::grant::{
    Grant, GrantRepository, ORGANIZATION_AGENT_ROLE, Principal, Scope,
};
use crate::application::{AgentRepository, AppRepository, JobRepository};
use crate::domain::clock;
use crate::domain::entities::{Agent, App, AppCredential, Job, JobId, OrganizationId};
use crate::domain::value_objects::app::{AppName, AppSecretHash, AppSecretLabel};
use crate::domain::value_objects::job::JobStatus;
use crate::domain::value_objects::role::RoleName;
use crate::infrastructure::persistence::postgres::{
    PgAppRepository, PgGrantRepository, PgJobRepository,
};
use crate::test_support::prelude::*;
use sqlx::PgPool;

const TEST_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$aGFzaGhhc2g";

fn default_credential(app: &App) -> AppCredential {
    AppCredential::create(
        app.id().clone(),
        AppSecretLabel::new("default").unwrap(),
        AppSecretHash::new(TEST_HASH).unwrap(),
    )
}

fn make_agent(org_id: &OrganizationId, name: &str) -> (App, AppCredential, Agent, Grant) {
    let app = App::create(org_id.clone(), AppName::new(name).unwrap());
    let credential = default_credential(&app);
    let agent = Agent::create(app.id().clone());
    let grant = Grant::new(
        Principal::App(app.id().clone()),
        RoleName::new(ORGANIZATION_AGENT_ROLE).unwrap(),
        Scope::Organization(org_id.clone()),
    );
    (app, credential, agent, grant)
}

#[sqlx::test(migrations = "../../migrations")]
async fn provision_agent_persists_app_row_and_grant(pool: PgPool) {
    let org = seed_org(&pool, "Acme").await;
    let app_repo = PgAppRepository::new(pool.clone());
    let agent_repo = PgAgentRepository::new(pool.clone());
    let (app, credential, agent, grant) = make_agent(org.id(), "ci-runner");

    app_repo
        .provision_agent(&app, &credential, &agent, &grant)
        .await
        .expect("provision_agent");

    let found = agent_repo
        .find_by_app_id(app.id())
        .await
        .expect("agent row");
    assert_eq!(found.app_id(), app.id());
    assert!(found.last_seen().is_none(), "fresh agent never seen");

    let list = agent_repo.list_by_organization(org.id()).await.unwrap();
    assert_eq!(list.len(), 1);

    let grants = PgGrantRepository::new(pool.clone())
        .list_all()
        .await
        .unwrap();
    assert!(
        grants
            .iter()
            .any(|g| g.principal == Principal::App(app.id().clone())),
        "agent grant minted with the app"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn plain_app_is_not_a_agent(pool: PgPool) {
    let org = seed_org(&pool, "Acme").await;
    let app_repo = PgAppRepository::new(pool.clone());
    let agent_repo = PgAgentRepository::new(pool.clone());

    let app = App::create(org.id().clone(), AppName::new("bot").unwrap());
    let credential = default_credential(&app);
    app_repo
        .create_app(&app, &credential)
        .await
        .expect("create_app");

    assert!(
        agent_repo.find_by_app_id(app.id()).await.is_err(),
        "a plain app has no agents row"
    );
    assert!(
        agent_repo
            .list_by_organization(org.id())
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn touch_last_seen_upserts_and_self_heals(pool: PgPool) {
    let org = seed_org(&pool, "Acme").await;
    let app_repo = PgAppRepository::new(pool.clone());
    let agent_repo = PgAgentRepository::new(pool.clone());

    // App exists but has no agents row yet (e.g. legacy / pre-migration agent).
    let app = App::create(org.id().clone(), AppName::new("legacy").unwrap());
    let credential = default_credential(&app);
    app_repo.create_app(&app, &credential).await.unwrap();

    let t1 = clock::now();
    agent_repo
        .touch_last_seen(app.id(), t1)
        .await
        .expect("self-heal insert");
    let after_first = agent_repo.find_by_app_id(app.id()).await.unwrap();
    assert!(after_first.last_seen().is_some());

    let t2 = t1 + chrono::Duration::seconds(30);
    agent_repo
        .touch_last_seen(app.id(), t2)
        .await
        .expect("upsert update");
    let after_second = agent_repo.find_by_app_id(app.id()).await.unwrap();
    assert_eq!(
        after_second.last_seen().unwrap().timestamp(),
        t2.timestamp()
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn agent_stats_aggregate_jobs_by_status(pool: PgPool) {
    let (org, _project, pipeline) = seed_org_project_pipeline(&pool, "stats").await;
    let app_repo = PgAppRepository::new(pool.clone());
    let agent_repo = PgAgentRepository::new(pool.clone());
    let job_repo = PgJobRepository::new(pool.clone());
    let (app, credential, agent, grant) = make_agent(org.id(), "runner");
    app_repo
        .provision_agent(&app, &credential, &agent, &grant)
        .await
        .unwrap();

    let mk = |status: JobStatus| {
        let now = clock::now();
        // Synthesize timestamps consistent with the status so the reconstructed
        // state is valid (a terminal job always has a finish time, etc.).
        let (started_at, finished_at) = match status {
            JobStatus::Pending => (None, None),
            JobStatus::Running => (Some(now), None),
            JobStatus::Completed
            | JobStatus::Failed
            | JobStatus::Cancelled
            | JobStatus::Orphaned => (Some(now - chrono::Duration::seconds(1)), Some(now)),
        };
        let state = crate::domain::entities::JobState::from_columns(status, started_at, finished_at)
            .unwrap();
        Job::from_persistence(
            JobId::generate(),
            pipeline.id().clone(),
            state,
            Some(app.id().clone()),
            vec![],
            vec![],
            crate::domain::value_objects::job::JobOrigin::App {
                app_id: app.id().clone(),
            },
            now,
            now,
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

    let stats = agent_repo.agent_stats(app.id()).await.unwrap();
    assert_eq!(stats.total, 4);
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.failed, 1);
    assert_eq!(stats.running, 1);
    assert!(stats.last_run_at.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_agent_keeps_jobs_and_nulls_attribution(pool: PgPool) {
    let (org, _project, pipeline) = seed_org_project_pipeline(&pool, "del").await;
    let app_repo = PgAppRepository::new(pool.clone());
    let agent_repo = PgAgentRepository::new(pool.clone());
    let job_repo = PgJobRepository::new(pool.clone());
    let (app, credential, agent, grant) = make_agent(org.id(), "runner");
    app_repo
        .provision_agent(&app, &credential, &agent, &grant)
        .await
        .unwrap();

    let mut job = Job::create_from_pipeline(
        &pipeline,
        crate::domain::value_objects::job::JobOrigin::App {
            app_id: app.id().clone(),
        },
    );
    job.assign_agent(app.id().clone());
    let job = job_repo.create(&job).await.unwrap();

    // Deleting the app cascades the agents row; the job survives with NULL.
    app_repo.delete(app.id()).await.unwrap();

    assert!(agent_repo.find_by_app_id(app.id()).await.is_err());
    let reloaded = job_repo.find_by_id(job.id()).await.expect("job survives");
    assert!(
        reloaded.agent_app_id().is_none(),
        "attribution nulled on agent delete"
    );
}
