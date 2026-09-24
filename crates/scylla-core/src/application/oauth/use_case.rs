use crate::application::oauth::provider::{OAuthProvider, PROVIDER_GITHUB};
use crate::application::oauth::repository::OAuthIdentityRepository;
use crate::application::signup::repository::SignupRepository;
use crate::application::{HashService, SessionRepository, UserRepository};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{OrganizationId, UserId};
use crate::domain::organization::Organization;
use crate::domain::organization::OrganizationName;
use crate::domain::role::RoleName;
use crate::domain::session::Session;
use crate::domain::user::User;
use crate::domain::user::{Password, Username};
use chrono::Duration;
use derive_more::Constructor;
use scylla_auth::authz::{Grant, ORGANIZATION_ADMIN_ROLE, PolicyControl, Principal, Scope};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

const SESSION_TTL_HOURS: i64 = 24;

pub struct OAuthOutcome {
    pub token: String,
    pub user_id: UserId,
    pub account: AccountOutcome,
}

pub enum AccountOutcome {
    New { organization_id: OrganizationId },
    Existing,
}

#[allow(clippy::too_many_arguments)]
#[derive(Constructor)]
pub struct OAuthUseCases {
    provider: Arc<dyn OAuthProvider>,
    identity_repo: Arc<dyn OAuthIdentityRepository>,
    signup_repo: Arc<dyn SignupRepository>,
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    hash_service: Arc<dyn HashService>,
    policy_control: Arc<dyn PolicyControl>,
}

impl OAuthUseCases {
    pub fn authorize_url(&self, state: &str) -> DomainResult<String> {
        self.provider.authorize_url(state)
    }

    #[instrument(skip(self, code))]
    pub async fn callback(&self, code: &str) -> DomainResult<OAuthOutcome> {
        let info = self.provider.exchange_code(code).await?;

        if let Some(user_id) = self
            .identity_repo
            .find_user_id(PROVIDER_GITHUB, &info.provider_user_id)
            .await?
        {
            // A deactivated account must not log back in through OAuth.
            let user = self.user_repo.find_by_id(&user_id).await?;
            if !user.is_active() {
                return Err(DomainError::unauthorized("User account is inactive"));
            }
            let token = self.issue_session(&user_id).await?;
            return Ok(OAuthOutcome {
                token,
                user_id,
                account: AccountOutcome::Existing,
            });
        }

        if let Some(email) = &info.email {
            if let Ok(user) = self.user_repo.find_by_email(email).await {
                if !user.is_active() {
                    return Err(DomainError::unauthorized("User account is inactive"));
                }
                self.identity_repo
                    .link(user.id(), PROVIDER_GITHUB, &info.provider_user_id)
                    .await?;
                let token = self.issue_session(user.id()).await?;
                return Ok(OAuthOutcome {
                    token,
                    user_id: user.id().clone(),
                    account: AccountOutcome::Existing,
                });
            }
        }

        let email = info.email.clone().ok_or_else(|| {
            DomainError::validation("GitHub account has no usable email for signup")
        })?;
        let username = Username::new(&info.login)?;
        let random = Password::new(Uuid::new_v4().to_string())?;
        let password_hash = self.hash_service.hash(&random).await?;
        let user = User::create(username, Some(email), password_hash);

        let org_name = OrganizationName::new(format!("{}'s organization", info.login))?;
        let organization = Organization::create(org_name, None)?;
        let role = RoleName::new(ORGANIZATION_ADMIN_ROLE)?;
        let grant = Grant::new(
            Principal::User(user.id().clone()),
            role,
            Scope::Organization(organization.id().clone()),
        );

        // One transaction: an account with no linked identity and no email would be unrecoverable.
        self.signup_repo
            .provision_account_with_identity(
                &user,
                &organization,
                &grant,
                PROVIDER_GITHUB,
                &info.provider_user_id,
            )
            .await?;
        self.policy_control.reload().await?;

        let token = self.issue_session(user.id()).await?;
        Ok(OAuthOutcome {
            token,
            user_id: user.id().clone(),
            account: AccountOutcome::New {
                organization_id: organization.id().clone(),
            },
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
