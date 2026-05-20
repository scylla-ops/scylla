use crate::application::permission::grant::{Grant, GrantScope, ORGANIZATION_ADMIN_ROLE};
use crate::application::permission::policy::PolicyControl;
use crate::application::signup::repository::SignupRepository;
use crate::application::{HashService, SessionRepository, UserRepository};
use crate::domain::entities::{Organization, OrganizationId, Session, User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::organization::OrganizationName;
use crate::domain::value_objects::role::name::RoleName;
use crate::domain::value_objects::user::{Email, Password, Username};
use async_trait::async_trait;
use chrono::Duration;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

pub const PROVIDER_GITHUB: &str = "github";
const SESSION_TTL_HOURS: i64 = 24;

/// Normalised identity returned by an OAuth provider after a successful code
/// exchange.
#[derive(Debug, Clone)]
pub struct OAuthUserInfo {
    pub provider_user_id: String,
    pub email: Option<Email>,
    pub login: String,
}

/// OAuth provider port (e.g. GitHub). The concrete HTTP implementation lives in
/// the infrastructure layer behind the `oauth-github` feature; tests stub it.
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    fn authorize_url(&self, state: &str) -> DomainResult<String>;
    async fn exchange_code(&self, code: &str) -> DomainResult<OAuthUserInfo>;
}

/// Persistence for external identity links (`user_oauth_identities`).
#[async_trait]
pub trait OAuthIdentityRepository: Send + Sync {
    async fn find_user_id(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<Option<UserId>>;
    async fn link(
        &self,
        user_id: &UserId,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<()>;
}

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
            return Ok(OAuthOutcome { token, user_id, organization_id: None });
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
        let grant = Grant::new(
            user.id().clone(),
            RoleName::new(ORGANIZATION_ADMIN_ROLE)?,
            GrantScope::Organization(organization.id().clone()),
        );

        self.signup_repo
            .provision_account(&user, &organization, &grant)
            .await?;
        self.policy_control.reload().await?;
        self.identity_repo
            .link(user.id(), PROVIDER_GITHUB, &info.provider_user_id)
            .await?;

        let token = self.issue_session(user.id()).await?;
        Ok(OAuthOutcome {
            token,
            user_id: user.id().clone(),
            organization_id: Some(organization.id().clone()),
        })
    }

    async fn issue_session(&self, user_id: &UserId) -> DomainResult<String> {
        let token = Uuid::new_v4().to_string();
        let session =
            Session::create(user_id.clone(), token.clone(), Duration::hours(SESSION_TTL_HOURS));
        self.session_repo.create(&session).await?;
        Ok(token)
    }
}
