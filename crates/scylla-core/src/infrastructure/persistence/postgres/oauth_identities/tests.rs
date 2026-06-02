use crate::application::UserOrganizationRepository;
use crate::application::audit::NoopAuditLog;
use crate::application::oauth::{OAuthProvider, OAuthUseCases, OAuthUserInfo};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::user::Email;
use crate::infrastructure::persistence::postgres::{
    PgAuthzEntityProvider, PgGrantRepository, PgOAuthIdentityRepository, PgPolicyRepository,
    PgSessionRepository, PgSignupRepository, PgUserOrganizationRepository, PgUserRepository,
};
use crate::infrastructure::{Argon2HashService, CedarPermissionService};
use crate::test_support::prelude::*;
use async_trait::async_trait;
use std::sync::Arc;

/// Returns a fixed identity, standing in for the GitHub HTTP exchange.
struct StubProvider {
    info: OAuthUserInfo,
}

#[async_trait]
impl OAuthProvider for StubProvider {
    fn authorize_url(&self, _state: &str) -> DomainResult<String> {
        Ok("https://github.test/authorize".to_string())
    }
    async fn exchange_code(&self, _code: &str) -> DomainResult<OAuthUserInfo> {
        Ok(self.info.clone())
    }
}

#[allow(clippy::type_complexity)]
async fn use_cases(
    pool: &sqlx::PgPool,
    info: OAuthUserInfo,
) -> OAuthUseCases<
    StubProvider,
    PgOAuthIdentityRepository,
    PgSignupRepository,
    PgUserRepository,
    PgSessionRepository,
    Argon2HashService,
    CedarPermissionService<PgAuthzEntityProvider>,
> {
    let permission = Arc::new(
        CedarPermissionService::new(
            Arc::new(PgAuthzEntityProvider::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool.clone())),
            Arc::new(PgPolicyRepository::new(pool.clone())),
            Arc::new(NoopAuditLog),
        )
        .await
        .expect("cedar"),
    );
    OAuthUseCases::new(
        Arc::new(StubProvider { info }),
        Arc::new(PgOAuthIdentityRepository::new(pool.clone())),
        Arc::new(PgSignupRepository::new(pool.clone())),
        Arc::new(PgUserRepository::new(pool.clone())),
        Arc::new(PgSessionRepository::new(pool.clone())),
        Arc::new(Argon2HashService::new()),
        permission,
    )
}

#[sqlx::test(migrations = "../../migrations")]
async fn first_login_provisions_account_then_second_reuses(pool: sqlx::PgPool) {
    let info = OAuthUserInfo {
        provider_user_id: "gh-123".to_string(),
        email: Some(Email::new("dev@example.com").unwrap()),
        login: "devuser".to_string(),
    };
    let uc = use_cases(&pool, info).await;

    let first = uc.callback("code-1").await.expect("first login");
    assert!(
        first.organization_id.is_some(),
        "new account gets an organization"
    );
    assert!(
        PgUserOrganizationRepository::new(pool.clone())
            .is_member(&first.user_id, first.organization_id.as_ref().unwrap())
            .await
            .unwrap()
    );

    let second = uc.callback("code-2").await.expect("second login");
    assert_eq!(
        first.user_id, second.user_id,
        "same identity reuses account"
    );
    assert!(
        second.organization_id.is_none(),
        "returning user creates no org"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_links_to_existing_user_by_email(pool: sqlx::PgPool) {
    // An existing account with the same email gets the identity linked.
    let existing = UserBuilder::new("legacy")
        .email("match@example.com")
        .build();
    crate::application::UserRepository::create(&PgUserRepository::new(pool.clone()), &existing)
        .await
        .expect("seed existing user");

    let info = OAuthUserInfo {
        provider_user_id: "gh-999".to_string(),
        email: Some(Email::new("match@example.com").unwrap()),
        login: "whatever".to_string(),
    };
    let uc = use_cases(&pool, info).await;

    let out = uc.callback("code").await.expect("login");
    assert_eq!(out.user_id, *existing.id(), "linked to existing account");
    assert!(
        out.organization_id.is_none(),
        "no new org for existing user"
    );
}
