use super::PgSignupRepository;
use crate::domain::role::RoleName;
use crate::postgres::{
    PgGrantRepository, PgOrganizationRepository, PgRoleRepository, PgUserRepository,
};
use crate::test_support::prelude::*;
use scylla_auth::authz::{Grant, GrantRepository, ORGANIZATION_ADMIN_ROLE, Principal, Scope};
use scylla_core::application::signup::repository::SignupRepository;
use scylla_core::application::{OrganizationRepository, UserRepository};
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

    let first_user = user("dup");
    let first_org = org("FirstOrg");
    let first_grant = org_admin_grant(first_user.id().clone(), first_org.id().clone());
    repo.provision_account(&first_user, &first_org, &first_grant)
        .await
        .expect("first provision");

    let clash_user = UserBuilder::new("dup").build();
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

#[sqlx::test(migrations = "../../migrations")]
async fn login_by_email_or_username(pool: PgPool) {
    use crate::domain::caller::CallerContext;
    use crate::domain::user::User;
    use crate::domain::user::{Email, Password, Username};
    use crate::postgres::PgSessionRepository;
    use scylla_core::application::auth::{AuthUseCases, Login};
    use scylla_core::application::{HashService, UserRepository};
    use scylla_core::infrastructure::Argon2HashService;
    use scylla_core::test_support::authz::DenyingPermissionService;
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

    let engine = actions(Arc::new(DenyingPermissionService::new()));
    let login = |identifier: &str, password: &str| {
        engine.run(
            &auth,
            &CallerContext::Anonymous,
            Login {
                identifier: identifier.to_string(),
                password: Password::new(password).unwrap(),
            },
        )
    };

    login("kevin@example.com", "SecurePass123!")
        .await
        .expect("login by email");
    login("kevin", "SecurePass123!")
        .await
        .expect("login by username");
    let err = login("kevin", "WrongPass123!")
        .await
        .expect_err("wrong password rejected");
    assert!(matches!(
        err,
        crate::domain::errors::DomainError::Unauthorized(_)
    ));
}

#[sqlx::test(migrations = "../../migrations")]
async fn signed_up_user_is_org_admin_of_own_org_only(pool: PgPool) {
    use crate::domain::caller::CallerContext;
    use crate::domain::organization::OrganizationName;
    use crate::domain::permission::Permission;
    use crate::domain::user::{Email, Password, Username};
    use crate::postgres::PgAuthzEntityProvider;
    use crate::postgres::PgSessionRepository;
    use scylla_auth::audit::NoopAuditLog;
    use scylla_auth::authz::PermissionService;
    use scylla_auth::cedar::CedarPermissionService;
    use scylla_core::application::SignupUseCases;
    use scylla_core::application::signup::Signup;
    use scylla_core::infrastructure::Argon2HashService;
    use std::sync::Arc;

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

    let outcome = actions(permission.clone())
        .run(
            &signup_uc,
            &CallerContext::Anonymous,
            Signup {
                username: Username::new("founder").unwrap(),
                email: Email::new("founder@example.com").unwrap(),
                password: Password::new("SecurePass123!").unwrap(),
                organization_name: OrganizationName::new("Founders Inc").unwrap(),
            },
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
