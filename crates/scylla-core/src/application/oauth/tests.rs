//! The OAuth flow's actions through the engine, on stub ports.

use super::*;
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::user::{Email, User};
use crate::test_support::authz::{DenyingPermissionService, actions};
use crate::test_support::stubs::{
    CountingPolicy, NoUsers, OneUser, StubHash, StubSessions, StubSignups,
};
use crate::test_support::users::UserBuilder;
use async_trait::async_trait;
use std::sync::Mutex;

struct StubProvider(OAuthUserInfo);

#[async_trait]
impl OAuthProvider for StubProvider {
    fn authorize_url(&self, state: &str) -> DomainResult<String> {
        Ok(format!("https://github.test/authorize?state={state}"))
    }
    async fn exchange_code(&self, _: &str) -> DomainResult<OAuthUserInfo> {
        Ok(self.0.clone())
    }
}

#[derive(Default)]
struct StubIdentities {
    known: Option<(String, UserId)>,
    linked: Mutex<Vec<(UserId, String)>>,
}

#[async_trait]
impl OAuthIdentityRepository for StubIdentities {
    async fn find_user_id(&self, _: &str, provider_user_id: &str) -> DomainResult<Option<UserId>> {
        Ok(self
            .known
            .as_ref()
            .filter(|(id, _)| id == provider_user_id)
            .map(|(_, user_id)| user_id.clone()))
    }
    async fn link(&self, user_id: &UserId, _: &str, provider_user_id: &str) -> DomainResult<()> {
        self.linked
            .lock()
            .unwrap()
            .push((user_id.clone(), provider_user_id.to_string()));
        Ok(())
    }
}

struct Lab {
    uc: OAuthUseCases,
    identities: Arc<StubIdentities>,
    signups: Arc<StubSignups>,
    sessions: Arc<StubSessions>,
    policy: Arc<CountingPolicy>,
}

impl Lab {
    async fn callback(&self) -> DomainResult<OAuthOutcome> {
        actions(Arc::new(DenyingPermissionService::new()))
            .run(
                &self.uc,
                &CallerContext::Anonymous,
                OAuthCallback {
                    code: "code".to_string(),
                },
            )
            .await
    }
}

fn info(email: Option<&str>) -> OAuthUserInfo {
    OAuthUserInfo {
        provider_user_id: "gh-1".to_string(),
        email: email.map(|e| Email::new(e).unwrap()),
        login: "dev".to_string(),
    }
}

fn lab(info: OAuthUserInfo, identities: StubIdentities, users: Arc<dyn UserRepository>) -> Lab {
    let identities = Arc::new(identities);
    let signups = Arc::new(StubSignups::default());
    let sessions = Arc::new(StubSessions::default());
    let policy = Arc::new(CountingPolicy::default());
    Lab {
        uc: OAuthUseCases::new(
            Arc::new(StubProvider(info)),
            identities.clone(),
            signups.clone(),
            users,
            sessions.clone(),
            Arc::new(StubHash::passwords()),
            policy.clone(),
        ),
        identities,
        signups,
        sessions,
        policy,
    }
}

fn dev(active: bool) -> User {
    UserBuilder::new("dev")
        .id(UserId::new("dev"))
        .email("dev@example.com")
        .is_active(active)
        .build()
}

#[tokio::test]
async fn a_first_login_provisions_an_account_with_the_identity() {
    let lab = lab(
        info(Some("dev@example.com")),
        StubIdentities::default(),
        Arc::new(NoUsers),
    );

    let outcome = lab.callback().await.unwrap();

    let AccountOutcome::New { organization_id } = &outcome.account else {
        panic!("a first login creates an organization");
    };
    let provisioned = lab.signups.provisioned();
    assert_eq!(provisioned.len(), 1);
    assert_eq!(provisioned[0].1.id(), organization_id);
    assert_eq!(provisioned[0].1.name().as_str(), "dev's organization");
    assert_eq!(provisioned[0].3.as_deref(), Some("gh-1"));
    assert_eq!(lab.policy.reloads(), 1);
    assert_eq!(lab.sessions.rows()[0].token(), outcome.token);
}

#[tokio::test]
async fn a_known_identity_signs_in_to_its_account() {
    let identities = StubIdentities {
        known: Some(("gh-1".to_string(), UserId::new("dev"))),
        ..StubIdentities::default()
    };
    let lab = lab(info(None), identities, Arc::new(OneUser(dev(true))));

    let outcome = lab.callback().await.unwrap();

    assert!(matches!(outcome.account, AccountOutcome::Existing));
    assert_eq!(outcome.user_id, UserId::new("dev"));
    assert!(lab.signups.provisioned().is_empty());
    assert_eq!(lab.policy.reloads(), 0);
}

#[tokio::test]
async fn a_new_identity_with_a_known_email_is_linked_to_that_account() {
    let lab = lab(
        info(Some("dev@example.com")),
        StubIdentities::default(),
        Arc::new(OneUser(dev(true))),
    );

    let outcome = lab.callback().await.unwrap();

    assert!(matches!(outcome.account, AccountOutcome::Existing));
    assert_eq!(
        *lab.identities.linked.lock().unwrap(),
        vec![(UserId::new("dev"), "gh-1".to_string())]
    );
    assert!(lab.signups.provisioned().is_empty());
}

#[tokio::test]
async fn an_inactive_account_cannot_sign_in_through_oauth() {
    let identities = StubIdentities {
        known: Some(("gh-1".to_string(), UserId::new("dev"))),
        ..StubIdentities::default()
    };
    let lab = lab(info(None), identities, Arc::new(OneUser(dev(false))));

    let err = lab.callback().await.err().unwrap();

    assert!(matches!(err, DomainError::Unauthorized(_)));
    assert!(lab.sessions.rows().is_empty());
}

#[tokio::test]
async fn a_new_identity_without_an_email_is_refused_before_any_write() {
    let lab = lab(info(None), StubIdentities::default(), Arc::new(NoUsers));

    let err = lab.callback().await.err().unwrap();

    assert!(matches!(err, DomainError::Validation(_)));
    assert!(lab.signups.provisioned().is_empty());
    assert!(lab.sessions.rows().is_empty());
}

#[tokio::test]
async fn the_auth_url_carries_the_state() {
    let lab = lab(info(None), StubIdentities::default(), Arc::new(NoUsers));

    let url = actions(Arc::new(DenyingPermissionService::new()))
        .run(
            &lab.uc,
            &CallerContext::Anonymous,
            GetAuthUrl {
                state: "s-1".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(url, "https://github.test/authorize?state=s-1");
}
