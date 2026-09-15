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
use scylla_auth::authz::{Grant, ORGANIZATION_ADMIN_ROLE, PolicyControl, Principal, Scope};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

const DEFAULT_SESSION_DURATION_HOURS: i64 = 24;

pub struct SignupOutcome {
    pub token: String,
    pub user_id: UserId,
    pub organization_id: OrganizationId,
}

/// No `CallerContext`: the one public entry point, so the Cedar-gated use cases are bypassed on purpose.
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

        let role = RoleName::new(ORGANIZATION_ADMIN_ROLE)?;
        let grant = Grant::new(
            Principal::User(user.id().clone()),
            role,
            Scope::Organization(organization.id().clone()),
        );

        self.signup_repo
            .provision_account(&user, &organization, &grant)
            .await?;

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
