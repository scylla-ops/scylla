use crate::application::authz::grant::{Grant, ORGANIZATION_ADMIN_ROLE, Principal, Scope};
use crate::application::authz::policy::PolicyControl;
use crate::application::signup::repository::SignupRepository;
use crate::application::{HashService, SessionRepository};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, UserId};
use crate::domain::organization::Organization;
use crate::domain::organization::OrganizationName;
use crate::domain::role::RoleName;
use crate::domain::session::Session;
use crate::domain::user::User;
use crate::domain::user::{Email, Password, Username};
use chrono::Duration;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

const DEFAULT_SESSION_DURATION_HOURS: i64 = 24;

/// What a successful signup returns to the transport layer.
pub struct SignupOutcome {
    pub token: String,
    pub user_id: UserId,
    pub organization_id: OrganizationId,
}

/// Self-service tenant onboarding. Unlike every other mutating use case this one
/// takes **no `CallerContext`**: signup is the single public, unauthenticated
/// entry point, so there is no principal to authorize. It deliberately bypasses
/// the Cedar-gated `UserUseCases`/`OrganizationUseCases` and writes the whole
/// account atomically through [`SignupRepository`], then makes the new
/// org-admin grant live via [`PolicyControl::reload`].
#[derive(Constructor)]
pub struct SignupUseCases<SR, S, H, PC>
where
    SR: SignupRepository,
    S: SessionRepository,
    H: HashService,
    PC: PolicyControl,
{
    signup_repo: Arc<SR>,
    session_repo: Arc<S>,
    hash_service: Arc<H>,
    policy_control: Arc<PC>,
}

impl<SR, S, H, PC> SignupUseCases<SR, S, H, PC>
where
    SR: SignupRepository,
    S: SessionRepository,
    H: HashService,
    PC: PolicyControl,
{
    #[instrument(skip_all, fields(username = %username, org = %organization_name))]
    pub async fn signup(
        &self,
        username: Username,
        email: Email,
        password: Password,
        organization_name: OrganizationName,
    ) -> DomainResult<SignupOutcome> {
        let password_hash = self.hash_service.hash(&password).await?;
        let user = User::create(username, Some(email), password_hash);
        let organization = Organization::create(organization_name, None)?;

        // The org creator becomes its admin via a scoped grant on their own org.
        let role = RoleName::new(ORGANIZATION_ADMIN_ROLE)?;
        let grant = Grant::new(
            Principal::User(user.id().clone()),
            role,
            Scope::Organization(organization.id().clone()),
        );

        self.signup_repo
            .provision_account(&user, &organization, &grant)
            .await?;

        // Rebuild the live policy set so the org-admin grant takes effect now.
        self.policy_control.reload().await?;

        let token = Uuid::new_v4().to_string();
        let session = Session::create(
            user.id().clone(),
            token.clone(),
            Duration::hours(DEFAULT_SESSION_DURATION_HOURS),
        );
        self.session_repo.create(&session).await?;

        Ok(SignupOutcome {
            token,
            user_id: user.id().clone(),
            organization_id: organization.id().clone(),
        })
    }
}
