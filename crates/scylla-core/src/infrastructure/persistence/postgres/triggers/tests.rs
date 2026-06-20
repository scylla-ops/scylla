use super::PgTriggerRepository;
use crate::application::{PipelineRepository, TriggerRepository};
use crate::domain::entities::{Pipeline, Trigger};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::pipeline::EnvKey;
use crate::domain::value_objects::trigger::{
    CronSpec, TriggerInput, TriggerName, TriggerSource, WebhookSpec,
};
use crate::infrastructure::persistence::postgres::PgPipelineRepository;
use crate::test_support::prelude::*;
use sqlx::PgPool;

fn cron_trigger(pipeline: &Pipeline, name: &str) -> Trigger {
    Trigger::create(
        pipeline.id().clone(),
        TriggerName::new(name).unwrap(),
        TriggerSource::Cron(CronSpec::new("0 9 * * *").unwrap()),
        vec![],
    )
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn cron_trigger_round_trips_with_inputs(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "u").await;
    let repo = PgTriggerRepository::new(pool);

    let trigger = Trigger::create(
        pipeline.id().clone(),
        TriggerName::new("nightly").unwrap(),
        TriggerSource::Cron(CronSpec::new("0 9 * * 1-5").unwrap()),
        vec![TriggerInput::literal(EnvKey::new("RUN_MODE").unwrap(), "nightly")],
    )
    .unwrap();
    repo.create(&trigger).await.unwrap();

    let found = repo.find_by_id(trigger.id()).await.unwrap();
    assert_eq!(found.name().as_str(), "nightly");
    assert!(found.is_enabled());
    assert_eq!(found.inputs().len(), 1);
    match found.source() {
        TriggerSource::Cron(c) => assert_eq!(c.expression(), "0 9 * * 1-5"),
        TriggerSource::Webhook(_) => panic!("expected cron source"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn webhook_trigger_round_trips_with_json_pointer(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "w").await;
    let repo = PgTriggerRepository::new(pool);

    let trigger = Trigger::create(
        pipeline.id().clone(),
        TriggerName::new("on-push").unwrap(),
        TriggerSource::Webhook(WebhookSpec::new(Some("X-Hub-Signature-256".into())).unwrap()),
        vec![TriggerInput::json_pointer(EnvKey::new("GIT_COMMIT").unwrap(), "/after").unwrap()],
    )
    .unwrap();
    repo.create(&trigger).await.unwrap();

    let found = repo.find_by_id(trigger.id()).await.unwrap();
    match found.source() {
        TriggerSource::Webhook(w) => {
            assert_eq!(w.signature_header(), Some("X-Hub-Signature-256"));
        }
        TriggerSource::Cron(_) => panic!("expected webhook source"),
    }
    assert_eq!(found.inputs().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_by_pipeline_filters(pool: PgPool) {
    let org = seed_org(&pool, "acme").await;
    let project = seed_project(&pool, &org, "rocket").await;
    let pipelines = PgPipelineRepository::new(pool.clone());
    let pa = pipeline(&project);
    let pb = pipeline(&project);
    pipelines.create(&pa).await.unwrap();
    pipelines.create(&pb).await.unwrap();
    let repo = PgTriggerRepository::new(pool);

    repo.create(&cron_trigger(&pa, "a")).await.unwrap();
    repo.create(&cron_trigger(&pa, "b")).await.unwrap();
    repo.create(&cron_trigger(&pb, "c")).await.unwrap();

    assert_eq!(repo.list_by_pipeline(pa.id()).await.unwrap().len(), 2);
    assert_eq!(repo.list_by_pipeline(pb.id()).await.unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_persists_changes(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "up").await;
    let repo = PgTriggerRepository::new(pool);

    let mut trigger = cron_trigger(&pipeline, "nightly");
    repo.create(&trigger).await.unwrap();

    trigger
        .update(
            TriggerName::new("midnight").unwrap(),
            TriggerSource::Cron(CronSpec::new("0 0 * * *").unwrap()),
            vec![],
        )
        .unwrap();
    repo.update(&trigger).await.unwrap();

    let found = repo.find_by_id(trigger.id()).await.unwrap();
    assert_eq!(found.name().as_str(), "midnight");
    match found.source() {
        TriggerSource::Cron(c) => assert_eq!(c.expression(), "0 0 * * *"),
        TriggerSource::Webhook(_) => panic!("expected cron source"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_enabled_persists(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "en").await;
    let repo = PgTriggerRepository::new(pool);

    let mut trigger = cron_trigger(&pipeline, "nightly");
    repo.create(&trigger).await.unwrap();

    trigger.set_enabled(false);
    repo.update(&trigger).await.unwrap();

    assert!(!repo.find_by_id(trigger.id()).await.unwrap().is_enabled());
}

#[sqlx::test(migrations = "../../migrations")]
async fn duplicate_name_in_pipeline_conflicts(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "dup").await;
    let repo = PgTriggerRepository::new(pool);

    repo.create(&cron_trigger(&pipeline, "dup")).await.unwrap();
    let err = repo.create(&cron_trigger(&pipeline, "dup")).await;
    assert!(matches!(err, Err(DomainError::Conflict(_))), "{err:?}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn cascade_pipeline_delete_removes_triggers(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "cas").await;
    let repo = PgTriggerRepository::new(pool.clone());

    let trigger = cron_trigger(&pipeline, "x");
    repo.create(&trigger).await.unwrap();

    PgPipelineRepository::new(pool)
        .delete(pipeline.id())
        .await
        .unwrap();

    assert!(matches!(
        repo.find_by_id(trigger.id()).await,
        Err(DomainError::NotFound { .. }),
    ));
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_then_find_returns_not_found(pool: PgPool) {
    let (_, _, pipeline) = seed_org_project_pipeline(&pool, "del").await;
    let repo = PgTriggerRepository::new(pool);

    let trigger = cron_trigger(&pipeline, "x");
    repo.create(&trigger).await.unwrap();

    repo.delete(trigger.id()).await.unwrap();
    assert!(matches!(
        repo.find_by_id(trigger.id()).await,
        Err(DomainError::NotFound { .. }),
    ));
}
