//! The invitation accept through the engine, on stub ports.

use super::*;
use crate::application::invitation::InvitationRepository;
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::InvitationId;
use crate::domain::invitation::Invitation;
use crate::domain::role::RoleName;
use crate::domain::user::{Email, Password, User, Username};
use crate::test_support::authz::{DenyingPermissionService, actions};
use crate::test_support::stubs::{CountingPolicy, NoUsers, OneUser, StubHash, StubSessions};
use crate::test_support::users::UserBuilder;
use async_trait::async_trait;
use scylla_auth::authz::{Grant, ORGANIZATION_MEMBER_ROLE, Principal, Scope};
use std::sync::Mutex;

struct OneInvitation {
    invitation: Invitation,
    accepted: Mutex<Vec<(Option<User>, UserId, Grant)>>,
}

#[async_trait]
impl InvitationRepository for OneInvitation {
    async fn create(&self, _: &Invitation) -> DomainResult<()> {
        unreachable!("no invitation create in an accept")
    }
    async fn find_by_id(&self, _: &InvitationId) -> DomainResult<Invitation> {
        unreachable!("an accept reads by token")
    }
    async fn find_by_token(&self, token: &str) -> DomainResult<Invitation> {
        if token == self.invitation.token() {
            Ok(self.invitation.clone())
        } else {
            Err(DomainError::not_found("Invitation", token))
        }
    }
    async fn list_pending(&self, _: &OrganizationId) -> DomainResult<Vec<Invitation>> {
        unreachable!("no listing in an accept")
    }
    async fn revoke(&self, _: &InvitationId) -> DomainResult<()> {
        unreachable!("no revoke in an accept")
    }
    async fn accept_atomic(
        &self,
        _: &InvitationId,
        new_user: Option<&User>,
        user_id: &UserId,
        grant: &Grant,
    ) -> DomainResult<()> {
        self.accepted
            .lock()
            .unwrap()
            .push((new_user.cloned(), user_id.clone(), grant.clone()));
        Ok(())
    }
}

struct Lab {
    uc: InvitationAcceptUseCases,
    invitations: Arc<OneInvitation>,
    sessions: Arc<StubSessions>,
    policy: Arc<CountingPolicy>,
    hash: Arc<StubHash>,
}

impl Lab {
    async fn accept(&self, token: &str) -> DomainResult<AcceptOutcome> {
        actions(Arc::new(DenyingPermissionService::new()))
            .run(
                &self.uc,
                &CallerContext::Anonymous,
                AcceptInvitation {
                    token: token.to_string(),
                    username: Username::new("newbie").unwrap(),
                    password: Password::new("SecurePass123!").unwrap(),
                },
            )
            .await
    }
}

fn lab(role: Option<&str>, users: Arc<dyn UserRepository>) -> Lab {
    let invitations = Arc::new(OneInvitation {
        invitation: Invitation::create(
            OrganizationId::new("org-1"),
            Email::new("newbie@example.com").unwrap(),
            role.map(|r| RoleName::new(r).unwrap()),
            UserId::new("boss"),
            "invite-token".to_string(),
        ),
        accepted: Mutex::default(),
    });
    let sessions = Arc::new(StubSessions::default());
    let policy = Arc::new(CountingPolicy::default());
    let hash = Arc::new(StubHash::passwords());
    Lab {
        uc: InvitationAcceptUseCases::new(
            invitations.clone(),
            users,
            hash.clone(),
            sessions.clone(),
            policy.clone(),
        ),
        invitations,
        sessions,
        policy,
        hash,
    }
}

#[tokio::test]
async fn an_accept_creates_the_invitee_grants_the_role_and_signs_in() {
    let lab = lab(Some("organization-admin"), Arc::new(NoUsers));

    let outcome = lab.accept("invite-token").await.unwrap();

    let accepted = lab.invitations.accepted.lock().unwrap();
    let (new_user, user_id, grant) = &accepted[0];
    assert_eq!(new_user.as_ref().map(User::id), Some(&outcome.user_id));
    assert_eq!(user_id, &outcome.user_id);
    assert_eq!(grant.principal, Principal::User(outcome.user_id.clone()));
    assert_eq!(grant.role.as_str(), "organization-admin");
    assert_eq!(
        grant.scope,
        Scope::Organization(OrganizationId::new("org-1"))
    );
    assert_eq!(outcome.organization_id, OrganizationId::new("org-1"));
    assert_eq!(lab.hash.hashed(), 1);
    assert_eq!(lab.policy.reloads(), 1);
    assert_eq!(lab.sessions.rows()[0].token(), outcome.token);
}

#[tokio::test]
async fn an_existing_account_joins_as_a_member_without_a_new_user() {
    let existing = UserBuilder::new("old")
        .id(UserId::new("old"))
        .email("newbie@example.com")
        .build();
    let lab = lab(None, Arc::new(OneUser(existing)));

    let outcome = lab.accept("invite-token").await.unwrap();

    assert_eq!(outcome.user_id, UserId::new("old"));
    let accepted = lab.invitations.accepted.lock().unwrap();
    assert!(accepted[0].0.is_none());
    assert_eq!(accepted[0].2.role.as_str(), ORGANIZATION_MEMBER_ROLE);
    assert_eq!(lab.hash.hashed(), 0);
}

#[tokio::test]
async fn an_unknown_token_writes_nothing() {
    let lab = lab(None, Arc::new(NoUsers));

    let err = lab.accept("no-such-token").await.err().unwrap();

    assert!(matches!(err, DomainError::NotFound { .. }));
    assert!(lab.invitations.accepted.lock().unwrap().is_empty());
    assert!(lab.sessions.rows().is_empty());
    assert_eq!(lab.policy.reloads(), 0);
}
