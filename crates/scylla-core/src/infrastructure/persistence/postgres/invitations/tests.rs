use crate::application::audit::NoopAuditLog;
use crate::application::caller::CallerContext;
use crate::application::invitation::InvitationUseCases;
use crate::application::permission::grant::{GrantPrincipal, GrantRepository, GrantScope};
use crate::application::{Mailer, NoopMailer, UserOrganizationRepository, UserRoleRepository};
use crate::domain::value_objects::role::name::RoleName;
use crate::domain::value_objects::user::{Email, Password, Username};
use crate::infrastructure::persistence::postgres::{
    PgAuthzEntityProvider, PgGrantRepository, PgInvitationRepository, PgOrganizationRepository,
    PgSessionRepository, PgUserOrganizationRepository, PgUserRepository, PgUserRoleRepository,
};
use crate::infrastructure::{Argon2HashService, CedarPermissionService};
use crate::test_support::prelude::*;
use std::sync::Arc;

#[allow(clippy::type_complexity)]
async fn use_cases(
    pool: &sqlx::PgPool,
) -> InvitationUseCases<
    PgInvitationRepository,
    CedarPermissionService<PgAuthzEntityProvider>,
    PgOrganizationRepository,
    PgUserRepository,
    Argon2HashService,
    PgSessionRepository,
    CedarPermissionService<PgAuthzEntityProvider>,
> {
    let permission = Arc::new(
        CedarPermissionService::new(
            Arc::new(PgAuthzEntityProvider::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool.clone())),
            Arc::new(crate::infrastructure::persistence::postgres::PgPolicyRepository::new(
                pool.clone(),
            )),
            Arc::new(NoopAuditLog),
        )
        .await
        .expect("cedar"),
    );
    let mailer: Arc<dyn Mailer> = Arc::new(NoopMailer);
    InvitationUseCases::new(
        Arc::new(PgInvitationRepository::new(pool.clone())),
        permission.clone(),
        mailer,
        Arc::new(PgOrganizationRepository::new(pool.clone())),
        Arc::new(PgUserRepository::new(pool.clone())),
        Arc::new(Argon2HashService::new()),
        Arc::new(PgSessionRepository::new(pool.clone())),
        permission,
    )
}

#[sqlx::test(migrations = "../../migrations")]
async fn invite_then_accept_joins_org_with_grant(pool: sqlx::PgPool) {
    let org = seed_org(&pool, "Acme").await;
    let inviter = seed_user(&pool, "boss").await;
    // Make the inviter a global admin so the AddOrganizationMember check passes.
    PgUserRoleRepository::new(pool.clone())
        .assign(inviter.id(), &RoleName::new("admin").unwrap())
        .await
        .expect("assign admin");

    let uc = use_cases(&pool).await;
    let caller = CallerContext::User(inviter.id().clone());

    let invite = uc
        .create_invite(
            &caller,
            org.id().clone(),
            Email::new("newbie@example.com").unwrap(),
            Some(RoleName::new("organization-admin").unwrap()),
        )
        .await
        .expect("create invite");

    let outcome = uc
        .accept(
            invite.token(),
            Username::new("newbie").unwrap(),
            Password::new("SecurePass123!").unwrap(),
        )
        .await
        .expect("accept invite");

    assert_eq!(outcome.organization_id, *org.id());
    assert!(
        PgUserOrganizationRepository::new(pool.clone())
            .is_member(&outcome.user_id, org.id())
            .await
            .unwrap(),
        "accepted user must be a member"
    );
    let grants = PgGrantRepository::new(pool).list_all().await.unwrap();
    assert!(
        grants.iter().any(|g| g.principal == GrantPrincipal::User(outcome.user_id.clone())
            && g.scope == GrantScope::Organization(org.id().clone())),
        "org-admin grant must be minted on accept"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn accept_with_unknown_token_fails(pool: sqlx::PgPool) {
    let uc = use_cases(&pool).await;
    let res = uc
        .accept(
            "no-such-token",
            Username::new("ghost").unwrap(),
            Password::new("SecurePass123!").unwrap(),
        )
        .await;
    assert!(res.is_err());
}
