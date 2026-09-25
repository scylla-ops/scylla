//! The signup's actions through the engine, on stub ports.

use super::*;
use crate::domain::caller::CallerContext;
use crate::domain::user::{Email, Password, Username};
use crate::test_support::authz::{DenyingPermissionService, actions};
use crate::test_support::stubs::{CountingPolicy, StubHash, StubSessions, StubSignups};
use scylla_auth::authz::Principal;

#[tokio::test]
async fn a_signup_provisions_the_account_reloads_and_signs_in_without_asking_a_permission() {
    let signups = Arc::new(StubSignups::default());
    let sessions = Arc::new(StubSessions::default());
    let hash = Arc::new(StubHash::passwords());
    let policy = Arc::new(CountingPolicy::default());
    let uc = SignupUseCases::new(
        signups.clone(),
        sessions.clone(),
        hash.clone(),
        policy.clone(),
    );

    let outcome = actions(Arc::new(DenyingPermissionService::new()))
        .run(
            &uc,
            &CallerContext::Anonymous,
            Signup {
                username: Username::new("founder").unwrap(),
                email: Email::new("founder@example.com").unwrap(),
                password: Password::new("SecurePass123!").unwrap(),
                organization_name: OrganizationName::new("Founders Inc").unwrap(),
            },
        )
        .await
        .unwrap();

    let provisioned = signups.provisioned();
    assert_eq!(provisioned.len(), 1);
    let (user, organization, grant, identity) = &provisioned[0];
    assert_eq!(user.id(), &outcome.user_id);
    assert_eq!(organization.id(), &outcome.organization_id);
    assert_eq!(organization.name().as_str(), "Founders Inc");
    assert_eq!(grant.principal, Principal::User(outcome.user_id.clone()));
    assert_eq!(grant.role.as_str(), ORGANIZATION_ADMIN_ROLE);
    assert!(identity.is_none());
    assert_eq!(hash.hashed(), 1);
    assert_eq!(policy.reloads(), 1);
    assert_eq!(sessions.rows()[0].token(), outcome.token);
}
