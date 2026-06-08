use crate::application::authz::grant::{Grant, Principal, Scope};
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::{
    DefaultRoleBindingRepository, DefaultRoleSlot, resolve_default_role,
};
use crate::application::oauth::provider::{OAuthProvider, PROVIDER_GITHUB};
use crate::application::oauth::repository::OAuthIdentityRepository;
use crate::application::signup::repository::SignupRepository;
use crate::application::{HashService, SessionRepository, UserRepository};
use crate::domain::entities::{Organization, OrganizationId, Session, User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::organization::OrganizationName;
use crate::domain::value_objects::user::{Password, Username};
use chrono::Duration;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

const SESSION_TTL_HOURS: i64 = 24;

pub struct OAuthOutcome {
    pub token: String,
    pub user_id: UserId,
    /// Set only when a brand-new account (and its organization) was created.
    pub organization_id: Option<OrganizationId>,
}

/// GitHub OAuth login. Public flow: the frontend redirects to `authorize_url`,
/// GitHub calls back with a code, and `callback` resolves it to a session —
/// linking to an existing account (by identity, then email) or provisioning a
/// new account + organization on first login.
// The derived `new` wires eight collaborators (the public onboarding flow needs
// them all); that's intrinsic, not a smell worth a parameter object here.
#[allow(clippy::too_many_arguments)]
#[derive(Constructor)]
pub struct OAuthUseCases<P, IR, SR, U, S, H, PC>
where
    P: OAuthProvider,
    IR: OAuthIdentityRepository,
    SR: SignupRepository,
    U: UserRepository,
    S: SessionRepository,
    H: HashService,
    PC: PolicyControl,
{
    provider: Arc<P>,
    identity_repo: Arc<IR>,
    signup_repo: Arc<SR>,
    user_repo: Arc<U>,
    session_repo: Arc<S>,
    hash_service: Arc<H>,
    policy_control: Arc<PC>,
    default_roles: Arc<dyn DefaultRoleBindingRepository>,
}

impl<P, IR, SR, U, S, H, PC> OAuthUseCases<P, IR, SR, U, S, H, PC>
where
    P: OAuthProvider,
    IR: OAuthIdentityRepository,
    SR: SignupRepository,
    U: UserRepository,
    S: SessionRepository,
    H: HashService,
    PC: PolicyControl,
{
    pub fn authorize_url(&self, state: &str) -> DomainResult<String> {
        self.provider.authorize_url(state)
    }

    #[instrument(skip(self, code))]
    pub async fn callback(&self, code: &str) -> DomainResult<OAuthOutcome> {
        let info = self.provider.exchange_code(code).await?;

        // 1. Known identity → straight to a session.
        if let Some(user_id) = self
            .identity_repo
            .find_user_id(PROVIDER_GITHUB, &info.provider_user_id)
            .await?
        {
            let token = self.issue_session(&user_id).await?;
            return Ok(OAuthOutcome {
                token,
                user_id,
                organization_id: None,
            });
        }

        // 2. Same email as an existing account → link the identity.
        if let Some(email) = &info.email {
            if let Ok(user) = self.user_repo.find_by_email(email).await {
                self.identity_repo
                    .link(user.id(), PROVIDER_GITHUB, &info.provider_user_id)
                    .await?;
                let token = self.issue_session(user.id()).await?;
                return Ok(OAuthOutcome {
                    token,
                    user_id: user.id().clone(),
                    organization_id: None,
                });
            }
        }

        // 3. Brand new → provision account + organization (like signup).
        let email = info.email.clone().ok_or_else(|| {
            DomainError::validation("GitHub account has no usable email for signup")
        })?;
        let username = Username::new(&info.login)?;
        // OAuth accounts have no password; store a random one (login is via OAuth).
        let random = Password::new(Uuid::new_v4().to_string())?;
        let password_hash = self.hash_service.hash(&random).await?;
        let user = User::create(username, Some(email), password_hash);

        let org_name = OrganizationName::new(format!("{}'s organization", info.login))?;
        let organization = Organization::create(org_name, None)?;
        let role =
            resolve_default_role(self.default_roles.as_ref(), DefaultRoleSlot::OrgCreation).await?;
        let grant = Grant::new(
            Principal::User(user.id().clone()),
            role,
            Scope::Organization(organization.id().clone()),
        );

        // Account, org, owner grant AND the GitHub identity link commit in ONE
        // transaction — a failure can't leave an account with no linked identity
        // (which, for an emailless GitHub account, would be unrecoverable).
        self.signup_repo
            .provision_account_with_identity(
                &user,
                &organization,
                &grant,
                PROVIDER_GITHUB,
                &info.provider_user_id,
            )
            .await?;
        // Make the owner grant live now (after the account is durably committed).
        self.policy_control.reload().await?;

        let token = self.issue_session(user.id()).await?;
        Ok(OAuthOutcome {
            token,
            user_id: user.id().clone(),
            organization_id: Some(organization.id().clone()),
        })
    }

    async fn issue_session(&self, user_id: &UserId) -> DomainResult<String> {
        let token = Uuid::new_v4().to_string();
        let session = Session::create(
            user_id.clone(),
            token.clone(),
            Duration::hours(SESSION_TTL_HOURS),
        );
        self.session_repo.create(&session).await?;
        Ok(token)
    }
}
