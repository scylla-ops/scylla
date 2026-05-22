use super::PgAppRepository;
use crate::application::app::AppRepository;
use crate::application::permission::grant::{
    Grant, GrantPrincipal, GrantRepository, GrantScope, AGENT_ROLE,
};
use crate::domain::entities::{App, AppCredential};
use crate::domain::value_objects::app::{AppName, AppSecretHash, AppSecretLabel};
use crate::domain::value_objects::role::name::RoleName;
use crate::infrastructure::persistence::postgres::{PgGrantRepository, PgOrganizationRepository};
use crate::test_support::prelude::*;
use sqlx::PgPool;

const TEST_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$aGFzaGhhc2g";

fn agent_app(
    org_id: &crate::domain::entities::OrganizationId,
    name: &str,
) -> (App, AppCredential, Grant) {
    let app = App::create(org_id.clone(), AppName::new(name).unwrap());
    let credential = AppCredential::create(
        app.id().clone(),
        AppSecretLabel::new("default").unwrap(),
        AppSecretHash::new(TEST_HASH).unwrap(),
    );
    let grant = Grant::new(
        GrantPrincipal::App(app.id().clone()),
        RoleName::new(AGENT_ROLE).unwrap(),
        GrantScope::Organization(org_id.clone()),
    );
    (app, credential, grant)
}

#[sqlx::test(migrations = "../../migrations")]
async fn provision_then_find_list_and_delete(pool: PgPool) {
    let org = seed_org(&pool, "Acme").await;
    let repo = PgAppRepository::new(pool.clone());
    let (app, credential, grant) = agent_app(org.id(), "ci-runner");

    repo.provision(&app, &credential, &grant).await.expect("provision");

    let found = repo.find_by_id(app.id()).await.expect("app persisted");
    assert_eq!(found.name().as_str(), "ci-runner");
    assert_eq!(found.organization_id(), org.id());

    let list = repo.list_by_organization(org.id()).await.unwrap();
    assert_eq!(list.len(), 1);

    // The initial agent grant is persisted in the same transaction.
    let grants = PgGrantRepository::new(pool.clone()).list_all().await.unwrap();
    assert!(
        grants
            .iter()
            .any(|g| g.principal == GrantPrincipal::App(app.id().clone())),
        "agent grant must be minted with the app"
    );

    repo.delete(app.id()).await.expect("delete");
    assert!(repo.find_by_id(app.id()).await.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn duplicate_name_in_same_org_conflicts(pool: PgPool) {
    let org = seed_org(&pool, "Acme").await;
    let repo = PgAppRepository::new(pool.clone());
    let (first, first_cred, first_grant) = agent_app(org.id(), "dup");
    repo.provision(&first, &first_cred, &first_grant).await.expect("first");

    let (clash, clash_cred, clash_grant) = agent_app(org.id(), "dup");
    let err = repo
        .provision(&clash, &clash_cred, &clash_grant)
        .await
        .expect_err("duplicate (org, name) must fail");
    assert!(matches!(
        err,
        crate::domain::errors::DomainError::Conflict(_)
    ));
}

#[sqlx::test(migrations = "../../migrations")]
async fn cascade_org_delete_removes_apps(pool: PgPool) {
    use crate::application::OrganizationRepository;

    let org = seed_org(&pool, "Acme").await;
    let repo = PgAppRepository::new(pool.clone());
    let (app, credential, grant) = agent_app(org.id(), "ci-runner");
    repo.provision(&app, &credential, &grant).await.expect("provision");

    PgOrganizationRepository::new(pool.clone())
        .delete(org.id())
        .await
        .expect("delete org");

    assert!(
        repo.find_by_id(app.id()).await.is_err(),
        "deleting the org cascades to its apps"
    );
}
