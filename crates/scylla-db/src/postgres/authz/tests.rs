use super::PgAuthzEntityProvider;
use crate::domain::ids::{SecretId, TriggerId};
use crate::domain::permission::ResourceRef;
use crate::domain::secret::{Secret, SecretName};
use crate::domain::trigger::{CronSpec, Trigger, TriggerName, TriggerSource};
use crate::postgres::{PgSecretRepository, PgTriggerRepository};
use crate::test_support::prelude::*;
use scylla_auth::authz::AuthzEntityProvider;
use scylla_core::application::TriggerRepository;
use scylla_core::application::secret::SecretRepository;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn a_job_resolves_to_its_pipeline_project_and_organization(pool: PgPool) {
    let (org, project, pipeline) = seed_org_project_pipeline(&pool, "job").await;
    let job = seed_job(&pool, &pipeline).await;

    let ancestors = PgAuthzEntityProvider::new(pool)
        .resource_ancestors(&ResourceRef::Job(job.id().clone()))
        .await
        .unwrap();

    assert_eq!(ancestors.organization.as_ref(), Some(org.id()));
    assert_eq!(ancestors.project.as_ref(), Some(project.id()));
    assert_eq!(ancestors.pipeline.as_ref(), Some(pipeline.id()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_secret_resolves_to_its_project_and_organization(pool: PgPool) {
    let (org, project, _) = seed_org_project_pipeline(&pool, "secret").await;
    let secret = Secret::create(
        project.id().clone(),
        SecretName::new("DB_PASSWORD").unwrap(),
        String::new(),
        vec![0xAA],
    );
    PgSecretRepository::new(pool.clone())
        .create(&secret)
        .await
        .unwrap();

    let ancestors = PgAuthzEntityProvider::new(pool)
        .resource_ancestors(&ResourceRef::Secret(secret.id().clone()))
        .await
        .unwrap();

    assert_eq!(ancestors.organization.as_ref(), Some(org.id()));
    assert_eq!(ancestors.project.as_ref(), Some(project.id()));
    assert!(ancestors.pipeline.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_secret_has_no_ancestors(pool: PgPool) {
    let ancestors = PgAuthzEntityProvider::new(pool)
        .resource_ancestors(&ResourceRef::Secret(SecretId::new("missing")))
        .await
        .unwrap();

    assert!(ancestors.organization.is_none());
    assert!(ancestors.project.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_trigger_resolves_to_its_pipeline_project_and_organization(pool: PgPool) {
    let (org, project, pipeline) = seed_org_project_pipeline(&pool, "trigger").await;
    let trigger = Trigger::create(
        pipeline.id().clone(),
        TriggerName::new("nightly").unwrap(),
        TriggerSource::Cron(CronSpec::new("0 9 * * *").unwrap()),
        vec![],
    )
    .unwrap();
    PgTriggerRepository::new(pool.clone())
        .create(&trigger, None)
        .await
        .unwrap();

    let ancestors = PgAuthzEntityProvider::new(pool)
        .resource_ancestors(&ResourceRef::Trigger(trigger.id().clone()))
        .await
        .unwrap();

    assert_eq!(ancestors.organization.as_ref(), Some(org.id()));
    assert_eq!(ancestors.project.as_ref(), Some(project.id()));
    assert_eq!(ancestors.pipeline.as_ref(), Some(pipeline.id()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_trigger_has_no_ancestors(pool: PgPool) {
    let ancestors = PgAuthzEntityProvider::new(pool)
        .resource_ancestors(&ResourceRef::Trigger(TriggerId::new("missing")))
        .await
        .unwrap();

    assert!(ancestors.organization.is_none());
    assert!(ancestors.project.is_none());
    assert!(ancestors.pipeline.is_none());
}
