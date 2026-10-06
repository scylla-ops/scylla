use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::InvitationId;
use crate::domain::invitation::{Invitation, InvitationStatus};
use crate::domain::role::RoleName;
use crate::domain::user::{Email, Password, User, Username};
use crate::postgres::{
    PgAuthzEntityProvider, PgGrantRepository, PgInvitationRepository, PgOrganizationRepository,
    PgRoleRepository, PgSessionRepository, PgUserRepository,
};
use crate::test_support::prelude::*;
use async_trait::async_trait;
use scylla_auth::audit::NoopAuditLog;
use scylla_auth::authz::{Grant, GrantRepository, Principal, Scope};
use scylla_auth::cedar::CedarPermissionService;
use scylla_core::application::Mailer;
use scylla_core::application::UserRepository;
use scylla_core::application::invitation::{
    AcceptInvitation, CreateInvitation, InvitationAcceptUseCases, InvitationRepository,
    InvitationUseCases,
};
use scylla_core::infrastructure::Argon2HashService;
use scylla_extension::Actions;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Inbox(Mutex<Vec<String>>);

#[async_trait]
impl Mailer for Inbox {
    async fn send(&self, _: &Email, _: &str, body: &str) -> DomainResult<()> {
        self.0.lock().unwrap().push(body.to_owned());
        Ok(())
    }
}

impl Inbox {
    fn token(&self) -> String {
        let body = self.0.lock().unwrap().last().cloned().unwrap();
        let (_, rest) = body.split_once("<code>").unwrap();
        rest.split_once("</code>").unwrap().0.to_owned()
    }
}

struct Lab {
    actions: Actions,
    invitations: InvitationUseCases,
    accept: InvitationAcceptUseCases,
    inbox: Arc<Inbox>,
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
    let inbox = Arc::new(Inbox::default());
    let invite_repo = Arc::new(PgInvitationRepository::new(pool.clone()));
    Lab {
        actions: actions(permission.clone()),
        invitations: InvitationUseCases::new(
            invite_repo.clone(),
            Arc::new(PgOrganizationRepository::new(pool.clone())),
            Arc::new(PgRoleRepository::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool.clone())),
            inbox.clone(),
        ),
        accept: InvitationAcceptUseCases::new(
            invite_repo,
            Arc::new(PgUserRepository::new(pool.clone())),
            Arc::new(Argon2HashService::new()),
            Arc::new(PgSessionRepository::new(pool.clone())),
        ),
        inbox,
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

    lab.actions
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
        .actions
        .run(
            &lab.accept,
            &CallerContext::Anonymous,
            AcceptInvitation {
                token: lab.inbox.token(),
                username: Username::new("newbie").unwrap(),
                password: Password::new("SecurePass123!").unwrap(),
            },
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
        .actions
        .run(
            &lab.accept,
            &CallerContext::Anonymous,
            AcceptInvitation {
                token: "no-such-token".to_string(),
                username: Username::new("ghost").unwrap(),
                password: Password::new("SecurePass123!").unwrap(),
            },
        )
        .await;
    assert!(res.is_err());
}

struct Pending {
    repo: PgInvitationRepository,
    invite: Invitation,
    invitee: User,
    grant: Grant,
}

async fn pending(pool: &sqlx::PgPool) -> Pending {
    let org = seed_org(pool, "Acme").await;
    let inviter = seed_user(pool, "boss").await;
    let invite = Invitation::create(
        org.id().clone(),
        Email::new("newbie@example.com").unwrap(),
        None,
        inviter.id().clone(),
    );
    let repo = PgInvitationRepository::new(pool.clone());
    repo.create(&invite, "token").await.expect("create invite");
    let invitee = user("newbie");
    let grant = Grant::new(
        Principal::User(invitee.id().clone()),
        RoleName::new("organization-member").unwrap(),
        Scope::Organization(org.id().clone()),
    );
    Pending {
        repo,
        invite,
        invitee,
        grant,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_accept_after_a_revoke_is_stale_and_writes_nothing(pool: sqlx::PgPool) {
    let p = pending(&pool).await;
    p.repo.revoke(p.invite.id()).await.expect("revoke");

    let err = p
        .repo
        .accept_atomic(p.invite.id(), Some(&p.invitee), p.invitee.id(), &p.grant)
        .await
        .expect_err("accept after revoke");

    assert!(matches!(err, DomainError::Stale(_)));
    assert!(matches!(
        PgUserRepository::new(pool.clone())
            .find_by_id(p.invitee.id())
            .await,
        Err(DomainError::NotFound(_))
    ));
    let grants = PgGrantRepository::new(pool).list_all().await.unwrap();
    assert!(!grants.iter().any(|g| g.id == p.grant.id));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_revoke_after_an_accept_is_stale_and_keeps_the_invitation_accepted(pool: sqlx::PgPool) {
    let p = pending(&pool).await;
    p.repo
        .accept_atomic(p.invite.id(), Some(&p.invitee), p.invitee.id(), &p.grant)
        .await
        .expect("accept");

    let err = p
        .repo
        .revoke(p.invite.id())
        .await
        .expect_err("revoke after accept");

    assert!(matches!(err, DomainError::Stale(_)));
    let stored = p.repo.find_by_id(p.invite.id()).await.unwrap();
    assert_eq!(stored.status(), InvitationStatus::Accepted);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_revoke_of_an_unknown_invitation_is_not_found(pool: sqlx::PgPool) {
    let err = PgInvitationRepository::new(pool)
        .revoke(&InvitationId::new("missing"))
        .await
        .expect_err("unknown invitation");

    assert!(matches!(err, DomainError::NotFound(_)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_store_keeps_a_digest_of_the_invite_token(pool: sqlx::PgPool) {
    let p = pending(&pool).await;

    let stored: String =
        sqlx::query_scalar("SELECT token_hash FROM organization_invites WHERE id = $1")
            .bind(p.invite.id().as_str())
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_ne!(stored, "token");
    assert!(p.repo.find_by_token(&stored).await.is_err());
    let found = p.repo.find_by_token("token").await.unwrap();
    assert_eq!(found.id(), p.invite.id());
}
