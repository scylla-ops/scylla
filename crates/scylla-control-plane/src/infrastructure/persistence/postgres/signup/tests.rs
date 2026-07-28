use super::PgSignupRepository;
use crate::application::authz::grant::{
    Grant, GrantRepository, ORGANIZATION_ADMIN_ROLE, Principal, Scope,
};
use crate::application::signup::repository::SignupRepository;
use crate::application::{OrganizationRepository, UserRepository};
use crate::domain::role::RoleName;
use crate::infrastructure::persistence::postgres::{
    PgGrantRepository, PgOrganizationRepository, PgRoleRepository, PgUserRepository,
};
use crate::test_support::prelude::*;
use sqlx::PgPool;

fn org_admin_grant(
    user_id: crate::domain::ids::UserId,
    org_id: crate::domain::ids::OrganizationId,
) -> Grant {
    Grant::new(
        Principal::User(user_id),
        RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
        Scope::Organization(org_id),
    )
}

#[sqlx::test(migrations = "../../migrations")]
async fn provision_account_persists_all_four_rows(pool: PgPool) {
    let repo = PgSignupRepository::new(pool.clone());
    let user = user("founder");
    let org = org("Acme");
    let grant = org_admin_grant(user.id().clone(), org.id().clone());

    repo.provision_account(&user, &org, &grant)
        .await
        .expect("provision");

    PgUserRepository::new(pool.clone())
        .find_by_id(user.id())
        .await
        .expect("user persisted");
    PgOrganizationRepository::new(pool.clone())
        .find_by_id(org.id())
        .await
        .expect("org persisted");
    let grants = PgGrantRepository::new(pool).list_all().await.unwrap();
    assert!(
        grants.iter().any(|g| g.id == grant.id),
        "org-admin grant persisted"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn username_conflict_rolls_back_the_whole_account(pool: PgPool) {
    let repo = PgSignupRepository::new(pool.clone());

    // First account succeeds.
    let first_user = user("dup");
    let first_org = org("FirstOrg");
    let first_grant = org_admin_grant(first_user.id().clone(), first_org.id().clone());
    repo.provision_account(&first_user, &first_org, &first_grant)
        .await
        .expect("first provision");

    // Second account reuses the username → unique violation mid-transaction.
    let clash_user = UserBuilder::new("dup").build(); // same username, fresh id
    let second_org = org("SecondOrg");
    let second_grant = org_admin_grant(clash_user.id().clone(), second_org.id().clone());
    let err = repo
        .provision_account(&clash_user, &second_org, &second_grant)
        .await
        .expect_err("username clash must fail");
    assert!(
        matches!(err, crate::domain::errors::DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );

    // Everything from the failed signup must be rolled back.
    let org_repo = PgOrganizationRepository::new(pool.clone());
    assert!(
        !org_repo.name_exists(second_org.name()).await.unwrap(),
        "second org must not be persisted"
    );
    let grants = PgGrantRepository::new(pool).list_all().await.unwrap();
    assert!(
        !grants.iter().any(|g| g.id == second_grant.id),
        "second grant must not be persisted"
    );
}

/// Login accepts either an email (contains `@`) or a username, and rejects a
/// wrong password with the same opaque error.
#[sqlx::test(migrations = "../../migrations")]
async fn login_by_email_or_username(pool: PgPool) {
    use crate::application::auth::use_case::AuthUseCases;
    use crate::application::{HashService, UserRepository};
    use crate::domain::user::User;
    use crate::domain::user::{Email, Password, Username};
    use crate::infrastructure::Argon2HashService;
    use crate::infrastructure::persistence::postgres::PgSessionRepository;
    use std::sync::Arc;

    let hash = Arc::new(Argon2HashService::new());
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let auth = AuthUseCases::new(
        user_repo.clone(),
        Arc::new(PgSessionRepository::new(pool.clone())),
        hash.clone(),
    );

    let password_hash = hash
        .hash(&Password::new("SecurePass123!").unwrap())
        .await
        .unwrap();
    let user = User::create(
        Username::new("kevin").unwrap(),
        Some(Email::new("kevin@example.com").unwrap()),
        password_hash,
    );
    user_repo.create(&user).await.expect("seed user");

    auth.login(
        "kevin@example.com".to_string(),
        Password::new("SecurePass123!").unwrap(),
    )
    .await
    .expect("login by email");
    auth.login(
        "kevin".to_string(),
        Password::new("SecurePass123!").unwrap(),
    )
    .await
    .expect("login by username");
    let err = auth
        .login("kevin".to_string(), Password::new("WrongPass123!").unwrap())
        .await
        .expect_err("wrong password rejected");
    assert!(matches!(
        err,
        crate::domain::errors::DomainError::Unauthorized(_)
    ));
}

/// End-to-end: signup through the use case must yield a user who is org-admin of
/// their own org (can update it) yet denied on any other org — the core tenant
/// isolation guarantee. Exercises the full Cedar path: signup links the grant,
/// reload makes it live, and a real `check` honours it.
#[sqlx::test(migrations = "../../migrations")]
async fn signed_up_user_is_org_admin_of_own_org_only(pool: PgPool) {
    use crate::application::audit::NoopAuditLog;
    use crate::application::caller::CallerContext;
    use crate::application::{PermissionService, SignupUseCases};
    use crate::domain::organization::OrganizationName;
    use crate::domain::permission::Permission;
    use crate::domain::user::{Email, Password, Username};
    use crate::infrastructure::persistence::postgres::PgSessionRepository;
    use crate::infrastructure::{Argon2HashService, CedarPermissionService, PgAuthzEntityProvider};
    use std::sync::Arc;

    // A second, foreign org the new user has nothing to do with.
    let foreign = seed_org(&pool, "Foreign Corp").await;

    let permission = Arc::new(
        CedarPermissionService::new(
            Arc::new(PgAuthzEntityProvider::new(pool.clone())),
            Arc::new(PgRoleRepository::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool.clone())),
            Arc::new(NoopAuditLog),
        )
        .await
        .expect("cedar service"),
    );
    let signup_uc = SignupUseCases::new(
        Arc::new(PgSignupRepository::new(pool.clone())),
        Arc::new(PgSessionRepository::new(pool.clone())),
        Arc::new(Argon2HashService::new()),
        permission.clone(),
    );

    let outcome = signup_uc
        .signup(
            Username::new("founder").unwrap(),
            Email::new("founder@example.com").unwrap(),
            Password::new("SecurePass123!").unwrap(),
            OrganizationName::new("Founders Inc").unwrap(),
        )
        .await
        .expect("signup");

    let caller = CallerContext::User(outcome.user_id.clone());

    permission
        .check(
            &caller,
            Permission::UpdateOrganization(outcome.organization_id.clone()),
        )
        .await
        .expect("org-admin can update own org");

    let err = permission
        .check(
            &caller,
            Permission::UpdateOrganization(foreign.id().clone()),
        )
        .await
        .expect_err("must be denied on a foreign org");
    assert!(
        matches!(err, crate::domain::errors::DomainError::Forbidden(_)),
        "expected Forbidden on cross-tenant access, got {err:?}"
    );
}
