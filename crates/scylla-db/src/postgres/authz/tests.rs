use super::PgAuthzEntityProvider;
use crate::domain::app::{App, AppCredential, AppName, AppSecretHash, AppSecretLabel};
use crate::domain::ids::{AppCredentialId, GrantId, InvitationId, SecretId, TriggerId};
use crate::domain::invitation::Invitation;
use crate::domain::permission::ResourceRef;
use crate::domain::role::RoleName;
use crate::domain::secret::{Secret, SecretName};
use crate::domain::trigger::{CronSpec, Trigger, TriggerName, TriggerSource};
use crate::domain::user::Email;
use crate::postgres::{
    PgAppRepository, PgGrantRepository, PgInvitationRepository, PgSecretRepository,
    PgTriggerRepository,
};
use crate::test_support::prelude::*;
use scylla_auth::authz::{
    AuthzEntityProvider, Grant, GrantRepository, ORGANIZATION_VIEWER_ROLE, PROJECT_VIEWER_ROLE,
    Principal, ResourceAncestors, SYSTEM_ADMIN_ROLE, Scope,
};
use scylla_core::application::TriggerRepository;
use scylla_core::application::app::AppRepository;
use scylla_core::application::invitation::InvitationRepository;
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

#[sqlx::test(migrations = "../../migrations")]
async fn an_invitation_resolves_to_its_organization(pool: PgPool) {
    let org = seed_org(&pool, "invitation").await;
    let inviter = seed_user(&pool, "inviter").await;
    let invitation = Invitation::create(
        org.id().clone(),
        Email::new("newbie@example.com").unwrap(),
        None,
        inviter.id().clone(),
        "token".to_string(),
    );
    PgInvitationRepository::new(pool.clone())
        .create(&invitation)
        .await
        .unwrap();

    let ancestors = PgAuthzEntityProvider::new(pool)
        .resource_ancestors(&ResourceRef::Invitation(invitation.id().clone()))
        .await
        .unwrap();

    assert_eq!(ancestors.organization.as_ref(), Some(org.id()));
    assert!(ancestors.project.is_none());
    assert!(ancestors.pipeline.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_invitation_has_no_ancestors(pool: PgPool) {
    let ancestors = PgAuthzEntityProvider::new(pool)
        .resource_ancestors(&ResourceRef::Invitation(InvitationId::new("missing")))
        .await
        .unwrap();

    assert!(ancestors.organization.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_app_secret_resolves_to_its_app_and_organization(pool: PgPool) {
    let org = seed_org(&pool, "app-secret").await;
    let app = App::create(org.id().clone(), AppName::new("ci-bot").unwrap());
    let credential = AppCredential::create(
        app.id().clone(),
        AppSecretLabel::new("default").unwrap(),
        AppSecretHash::new("$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$aGFzaGhhc2g").unwrap(),
    );
    PgAppRepository::new(pool.clone())
        .create_app(&app, &credential)
        .await
        .unwrap();

    let ancestors = PgAuthzEntityProvider::new(pool)
        .resource_ancestors(&ResourceRef::AppSecret(credential.id().clone()))
        .await
        .unwrap();

    assert_eq!(ancestors.organization.as_ref(), Some(org.id()));
    assert_eq!(ancestors.app.as_ref(), Some(app.id()));
    assert!(ancestors.project.is_none());
    assert!(ancestors.pipeline.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_app_secret_has_no_ancestors(pool: PgPool) {
    let ancestors = PgAuthzEntityProvider::new(pool)
        .resource_ancestors(&ResourceRef::AppSecret(AppCredentialId::new("missing")))
        .await
        .unwrap();

    assert!(ancestors.organization.is_none());
    assert!(ancestors.app.is_none());
}

async fn grant_ancestors(pool: &PgPool, role: &str, scope: Scope) -> ResourceAncestors {
    let user = seed_user(pool, "grantee").await;
    let grant = Grant::new(
        Principal::User(user.id().clone()),
        RoleName::new(role).unwrap(),
        scope,
    );
    PgGrantRepository::new(pool.clone())
        .create(&grant)
        .await
        .unwrap();
    PgAuthzEntityProvider::new(pool.clone())
        .resource_ancestors(&ResourceRef::Grant(GrantId::new(grant.id)))
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_grant_resolves_to_its_project_and_organization(pool: PgPool) {
    let (org, project, _) = seed_org_project_pipeline(&pool, "grant").await;

    let ancestors = grant_ancestors(
        &pool,
        PROJECT_VIEWER_ROLE,
        Scope::Project(project.id().clone()),
    )
    .await;

    assert_eq!(ancestors.organization.as_ref(), Some(org.id()));
    assert_eq!(ancestors.project.as_ref(), Some(project.id()));
    assert!(ancestors.pipeline.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_organization_grant_resolves_to_its_organization(pool: PgPool) {
    let org = seed_org(&pool, "grant").await;

    let ancestors = grant_ancestors(
        &pool,
        ORGANIZATION_VIEWER_ROLE,
        Scope::Organization(org.id().clone()),
    )
    .await;

    assert_eq!(ancestors.organization.as_ref(), Some(org.id()));
    assert!(ancestors.project.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_system_grant_has_no_ancestors(pool: PgPool) {
    let ancestors = grant_ancestors(&pool, SYSTEM_ADMIN_ROLE, Scope::System).await;

    assert!(ancestors.organization.is_none());
    assert!(ancestors.project.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_grant_has_no_ancestors(pool: PgPool) {
    let ancestors = PgAuthzEntityProvider::new(pool)
        .resource_ancestors(&ResourceRef::Grant(GrantId::new("missing")))
        .await
        .unwrap();

    assert!(ancestors.organization.is_none());
    assert!(ancestors.project.is_none());
}
