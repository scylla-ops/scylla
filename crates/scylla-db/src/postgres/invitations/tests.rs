use crate::domain::caller::CallerContext;
use crate::domain::role::RoleName;
use crate::domain::user::{Email, Password, Username};
use crate::postgres::{
    PgAuthzEntityProvider, PgGrantRepository, PgInvitationRepository, PgOrganizationRepository,
    PgRoleRepository, PgSessionRepository, PgUserRepository,
};
use crate::test_support::prelude::*;
use scylla_auth::audit::NoopAuditLog;
use scylla_auth::authz::{Grant, GrantRepository, Principal, Scope};
use scylla_auth::cedar::CedarPermissionService;
use scylla_core::application::invitation::{
    CreateInvitation, InvitationAcceptUseCases, InvitationUseCases,
};
use scylla_core::application::{Mailer, NoopMailer};
use scylla_core::infrastructure::Argon2HashService;
use scylla_extension::Actions;
use std::sync::Arc;

struct Lab {
    actions: Actions,
    invitations: InvitationUseCases,
    accept: InvitationAcceptUseCases,
}

async fn lab(pool: &sqlx::PgPool) -> Lab {
    let permission = Arc::new(
        CedarPermissionService::new(
            Arc::new(PgAuthzEntityProvider::new(pool.clone())),
            Arc::new(PgRoleRepository::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool.clone())),
            Arc::new(NoopAuditLog),
        )
        .await
        .expect("cedar"),
    );
    let mailer: Arc<dyn Mailer> = Arc::new(NoopMailer);
    let invite_repo = Arc::new(PgInvitationRepository::new(pool.clone()));
    Lab {
        actions: actions(permission.clone()),
        invitations: InvitationUseCases::new(
            invite_repo.clone(),
            Arc::new(PgOrganizationRepository::new(pool.clone())),
            Arc::new(PgRoleRepository::new(pool.clone())),
            mailer,
        ),
        accept: InvitationAcceptUseCases::new(
            invite_repo,
            Arc::new(PgUserRepository::new(pool.clone())),
            Arc::new(Argon2HashService::new()),
            Arc::new(PgSessionRepository::new(pool.clone())),
            permission,
        ),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn invite_then_accept_joins_org_with_grant(pool: sqlx::PgPool) {
    let org = seed_org(&pool, "Acme").await;
    let inviter = seed_user(&pool, "boss").await;
    PgGrantRepository::new(pool.clone())
        .create(&Grant::new(
            Principal::User(inviter.id().clone()),
            RoleName::new("system-admin").unwrap(),
            Scope::System,
        ))
        .await
        .expect("grant system-admin");

    let lab = lab(&pool).await;
    let caller = CallerContext::User(inviter.id().clone());

    let invite = lab
        .actions
        .run(
            &lab.invitations,
            &caller,
            CreateInvitation {
                organization_id: org.id().clone(),
                email: Email::new("newbie@example.com").unwrap(),
                role: Some(RoleName::new("organization-admin").unwrap()),
            },
        )
        .await
        .expect("create invite");

    let outcome = lab
        .accept
        .accept(
            invite.token(),
            Username::new("newbie").unwrap(),
            Password::new("SecurePass123!").unwrap(),
        )
        .await
        .expect("accept invite");

    assert_eq!(outcome.organization_id, *org.id());
    let grants = PgGrantRepository::new(pool).list_all().await.unwrap();
    assert!(
        grants
            .iter()
            .any(|g| g.principal == Principal::User(outcome.user_id.clone())
                && g.scope == Scope::Organization(org.id().clone())),
        "org-admin grant must be minted on accept"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn accept_with_unknown_token_fails(pool: sqlx::PgPool) {
    let lab = lab(&pool).await;
    let res = lab
        .accept
        .accept(
            "no-such-token",
            Username::new("ghost").unwrap(),
            Password::new("SecurePass123!").unwrap(),
        )
        .await;
    assert!(res.is_err());
}
