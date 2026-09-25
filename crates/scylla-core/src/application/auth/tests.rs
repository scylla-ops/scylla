//! The session's actions through the engine, on stub ports.

use super::*;
use crate::domain::app::{AppSecret, AppSecretHash};
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::user::{Password, PasswordHash};
use crate::test_support::authz::{DenyingPermissionService, actions};
use crate::test_support::sessions::SessionBuilder;
use crate::test_support::stubs::{OneUser, StubSessions};
use crate::test_support::users::UserBuilder;
use async_trait::async_trait;
use scylla_extension::Actions;

const PASSWORD: &str = "SecurePass123!";

struct CheckingHash;

#[async_trait]
impl HashService for CheckingHash {
    async fn hash(&self, _: &Password) -> DomainResult<PasswordHash> {
        unreachable!("no password to hash in a session action")
    }
    async fn verify(&self, password: &Password, _: &PasswordHash) -> DomainResult<bool> {
        Ok(password.as_str() == PASSWORD)
    }
    async fn hash_secret(&self, _: &AppSecret) -> DomainResult<AppSecretHash> {
        unreachable!("no app secret in a session action")
    }
    async fn verify_secret(&self, _: &AppSecret, _: &AppSecretHash) -> DomainResult<bool> {
        unreachable!("no app secret in a session action")
    }
}

struct Lab {
    actions: Actions,
    uc: AuthUseCases,
    sessions: Arc<StubSessions>,
}

impl Lab {
    async fn login(&self, identifier: &str, password: &str) -> DomainResult<Session> {
        let login = Login {
            identifier: identifier.to_string(),
            password: Password::new(password).unwrap(),
        };
        self.actions
            .run(&self.uc, &CallerContext::Anonymous, login)
            .await
    }

    async fn validate(&self, token: &str) -> bool {
        let query = ValidateToken {
            token: token.to_string(),
        };
        self.actions
            .run(&self.uc, &CallerContext::Anonymous, query)
            .await
            .unwrap()
    }
}

fn lab(active: bool, sessions: StubSessions) -> Lab {
    let user = UserBuilder::new("kevin")
        .id(UserId::new("kevin"))
        .email("kevin@example.com")
        .is_active(active)
        .build();
    let sessions = Arc::new(sessions);
    Lab {
        actions: actions(Arc::new(DenyingPermissionService::new())),
        uc: AuthUseCases::new(
            Arc::new(OneUser(user)),
            sessions.clone(),
            Arc::new(CheckingHash),
        ),
        sessions,
    }
}

#[tokio::test]
async fn a_login_by_username_or_email_stores_a_session_without_asking_a_permission() {
    let lab = lab(true, StubSessions::default());

    let by_name = lab.login("kevin", PASSWORD).await.unwrap();
    let by_email = lab.login("kevin@example.com", PASSWORD).await.unwrap();

    assert_eq!(by_name.user_id(), &UserId::new("kevin"));
    assert_ne!(by_name.token(), by_email.token());
    assert_eq!(lab.sessions.rows().len(), 2);
}

#[tokio::test]
async fn a_wrong_password_or_an_unknown_account_gives_one_opaque_error() {
    let lab = lab(true, StubSessions::default());

    for (identifier, password) in [("kevin", "WrongPass123!"), ("ghost", PASSWORD)] {
        let err = lab.login(identifier, password).await.unwrap_err();
        assert!(
            matches!(&err, DomainError::Unauthorized(m) if m == "Invalid username or password")
        );
    }
    assert!(lab.sessions.rows().is_empty());
}

#[tokio::test]
async fn an_inactive_account_cannot_log_in() {
    let lab = lab(false, StubSessions::default());

    let err = lab.login("kevin", PASSWORD).await.unwrap_err();

    assert!(matches!(&err, DomainError::Unauthorized(m) if m == "User account is inactive"));
    assert!(lab.sessions.rows().is_empty());
}

#[tokio::test]
async fn a_live_token_is_valid_and_an_expired_one_is_deleted() {
    let user_id = UserId::new("kevin");
    let live = lab(
        true,
        StubSessions::with(SessionBuilder::new(&user_id).token("live").build()),
    );
    let expired = lab(
        true,
        StubSessions::with(
            SessionBuilder::new(&user_id)
                .token("expired")
                .expired(true)
                .build(),
        ),
    );

    assert!(live.validate("live").await);
    assert!(!live.validate("unknown").await);
    assert!(!live.validate("").await);
    assert!(!expired.validate("expired").await);
    assert_eq!(expired.sessions.deleted(), vec!["expired".to_string()]);
}

#[tokio::test]
async fn a_revoke_deletes_the_session() {
    let user_id = UserId::new("kevin");
    let lab = lab(
        true,
        StubSessions::with(SessionBuilder::new(&user_id).token("t").build()),
    );

    lab.actions
        .run(
            &lab.uc,
            &CallerContext::Anonymous,
            RevokeToken {
                token: "t".to_string(),
            },
        )
        .await
        .unwrap();

    assert!(lab.sessions.rows().is_empty());
    assert_eq!(lab.sessions.deleted(), vec!["t".to_string()]);
}
