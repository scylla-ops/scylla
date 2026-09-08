use super::PgAppTokenRepository;
use crate::application::HashService;
use crate::application::app::{AppRepository, AppTokenRepository, AppTokenUseCases};
use crate::application::authz::grant::{Grant, ORGANIZATION_AGENT_ROLE, Principal, Scope};
use crate::domain::app::{App, AppCredential};
use crate::domain::app::{AppName, AppSecret, AppSecretLabel};
use crate::domain::role::RoleName;
use crate::infrastructure::Argon2HashService;
use crate::infrastructure::persistence::postgres::{PgAppCredentialRepository, PgAppRepository};
use crate::test_support::prelude::*;
use sqlx::PgPool;
use std::sync::Arc;

async fn seed_app(pool: &PgPool, secret: &AppSecret) -> App {
    let org = seed_org(pool, "Acme").await;
    let hash = Argon2HashService::new().hash_secret(secret).await.unwrap();
    let app = App::create(org.id().clone(), AppName::new("ci").unwrap());
    let credential = AppCredential::create(
        app.id().clone(),
        AppSecretLabel::new("default").unwrap(),
        hash,
    );
    let grant = Grant::new(
        Principal::App(app.id().clone()),
        RoleName::new(ORGANIZATION_AGENT_ROLE).unwrap(),
        Scope::Organization(org.id().clone()),
    );
    PgAppRepository::new(pool.clone())
        .provision(&app, &credential, &grant)
        .await
        .unwrap();
    app
}

fn use_cases(
    pool: &PgPool,
) -> AppTokenUseCases<
    PgAppRepository,
    PgAppTokenRepository,
    PgAppCredentialRepository,
    Argon2HashService,
> {
    AppTokenUseCases::new(
        Arc::new(PgAppRepository::new(pool.clone())),
        Arc::new(PgAppTokenRepository::new(pool.clone())),
        Arc::new(PgAppCredentialRepository::new(pool.clone())),
        Arc::new(Argon2HashService::new()),
    )
}

#[sqlx::test(migrations = "../../migrations")]
async fn issue_with_correct_secret_then_token_resolves(pool: PgPool) {
    let secret = crate::application::app::mint_app_secret();
    let app = seed_app(&pool, &secret).await;

    let outcome = use_cases(&pool)
        .issue(app.id().clone(), secret)
        .await
        .expect("issue");

    let found = PgAppTokenRepository::new(pool.clone())
        .find_by_token(&outcome.token)
        .await
        .expect("token persisted");
    assert_eq!(found.app_id(), app.id());
    assert!(!found.is_expired());
}

#[sqlx::test(migrations = "../../migrations")]
async fn issue_with_wrong_secret_is_unauthorized(pool: PgPool) {
    let secret = crate::application::app::mint_app_secret();
    let app = seed_app(&pool, &secret).await;

    let result = use_cases(&pool)
        .issue(app.id().clone(), crate::application::app::mint_app_secret())
        .await;
    assert!(matches!(
        result,
        Err(crate::domain::errors::DomainError::Unauthorized(_))
    ));
}
