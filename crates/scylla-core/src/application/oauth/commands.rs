//! The OAuth flow's writes. One block per command, in the order it runs: the struct, its
//! access, its payload types, what `Prepare` builds, what `Persist` writes.

use super::{AccountOutcome, OAuthOutcome, OAuthUseCases, PROVIDER_GITHUB};
use crate::application::auth::new_session;
use crate::application::signup::NewAccount;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::organization::OrganizationName;
use crate::domain::session::Session;
use crate::domain::user::{Password, User, Username};
use async_trait::async_trait;
use scylla_extension::{
    Access, Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};
use uuid::Uuid;

/// No `Debug`: `code` is exchanged for the provider's token.
pub struct OAuthCallback {
    pub code: String,
}

/// The account the provider identity signs in to: already linked, linked now by its email, or
/// created with the identity in one transaction.
pub enum OAuthAccount {
    Existing,
    Link {
        provider_user_id: String,
    },
    New {
        account: Box<NewAccount>,
        provider_user_id: String,
    },
}

pub struct OAuthSignIn {
    pub account: OAuthAccount,
    pub session: Session,
}

impl Describe for OAuthCallback {
    fn access(&self) -> Access {
        Access::Public
    }
}

impl Command for OAuthCallback {
    type Staged = Draft<OAuthSignIn>;
    type Committed = OAuthOutcome;
}

#[async_trait]
impl Run<Prepare<OAuthCallback>> for OAuthUseCases {
    async fn run(&self, input: Authorized<OAuthCallback>) -> DomainResult<Prepared<OAuthCallback>> {
        let info = self.provider.exchange_code(&input.command().code).await?;
        let sign_in = |account, user_id| {
            Ok(input.prepared(Draft::new(OAuthSignIn {
                account,
                session: new_session(user_id),
            })))
        };

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
            return sign_in(OAuthAccount::Existing, user_id);
        }

        if let Some(email) = &info.email {
            if let Ok(user) = self.user_repo.find_by_email(email).await {
                if !user.is_active() {
                    return Err(DomainError::unauthorized("User account is inactive"));
                }
                let provider_user_id = info.provider_user_id;
                return sign_in(OAuthAccount::Link { provider_user_id }, user.id().clone());
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
        let account = NewAccount::new(user, org_name)?;
        let user_id = account.user.id().clone();
        sign_in(
            OAuthAccount::New {
                account: Box::new(account),
                provider_user_id: info.provider_user_id,
            },
            user_id,
        )
    }
}

#[async_trait]
impl Run<Persist<OAuthCallback>> for OAuthUseCases {
    async fn run(&self, input: Prepared<OAuthCallback>) -> DomainResult<Committed<OAuthCallback>> {
        input
            .commit(async |draft| {
                let OAuthSignIn { account, session } = draft.into_inner();
                let user_id = session.user_id();
                let account = match account {
                    OAuthAccount::Existing => AccountOutcome::Existing,
                    OAuthAccount::Link { provider_user_id } => {
                        self.identity_repo
                            .link(user_id, PROVIDER_GITHUB, &provider_user_id)
                            .await?;
                        AccountOutcome::Existing
                    }
                    OAuthAccount::New {
                        account,
                        provider_user_id,
                    } => {
                        self.signup_repo
                            .provision_account_with_identity(
                                &account.user,
                                &account.organization,
                                &account.grant,
                                PROVIDER_GITHUB,
                                &provider_user_id,
                            )
                            .await?;
                        self.policy_control.reload().await?;
                        AccountOutcome::New {
                            organization_id: account.organization.id().clone(),
                        }
                    }
                };
                self.session_repo.create(&session).await?;
                Ok(OAuthOutcome {
                    token: session.token().to_string(),
                    user_id: user_id.clone(),
                    account,
                })
            })
            .await
    }
}
