use crate::application::invitation::repository::InvitationRepository;
use crate::application::{HashService, SessionRepository, UserRepository};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{OrganizationId, UserId};
use crate::domain::role::RoleName;
use crate::domain::session::Session;
use crate::domain::user::User;
use crate::domain::user::{Password, Username};
use chrono::Duration;
use derive_more::Constructor;
use scylla_auth::authz::{Grant, ORGANIZATION_MEMBER_ROLE, PolicyControl, Principal, Scope};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

const SESSION_TTL_HOURS: i64 = 24;

pub struct AcceptOutcome {
    pub token: String,
    pub user_id: UserId,
    pub organization_id: OrganizationId,
}

/// The token is the credential: the invitee has no account yet, so no permission is asked.
#[derive(Constructor)]
pub struct InvitationAcceptUseCases {
    invite_repo: Arc<dyn InvitationRepository>,
    user_repo: Arc<dyn UserRepository>,
    hash_service: Arc<dyn HashService>,
    session_repo: Arc<dyn SessionRepository>,
    policy_control: Arc<dyn PolicyControl>,
}

impl InvitationAcceptUseCases {
    #[instrument(skip_all, fields(username = %username))]
    pub async fn accept(
        &self,
        token: &str,
        username: Username,
        password: Password,
    ) -> DomainResult<AcceptOutcome> {
        let invite = self.invite_repo.find_by_token(token).await?;
        if !invite.is_acceptable() {
            return Err(DomainError::business_rule("Invitation is no longer valid"));
        }

        let (new_user, user_id) =
            if let Ok(existing) = self.user_repo.find_by_email(invite.email()).await {
                (None, existing.id().clone())
            } else {
                let password_hash = self.hash_service.hash(&password).await?;
                let user = User::create(username, Some(invite.email().clone()), password_hash);
                let id = user.id().clone();
                (Some(user), id)
            };

        // The grant is the join: a roleless invite still mints `organization-member`.
        let role = match invite.role() {
            Some(role) => role.clone(),
            None => RoleName::new(ORGANIZATION_MEMBER_ROLE)?,
        };
        let grant = Grant::new(
            Principal::User(user_id.clone()),
            role,
            Scope::Organization(invite.organization_id().clone()),
        );

        self.invite_repo
            .accept_atomic(invite.id(), new_user.as_ref(), &user_id, &grant)
            .await?;

        self.policy_control.reload().await?;

        let session_token = Uuid::new_v4().to_string();
        let session = Session::create(
            user_id.clone(),
            session_token.clone(),
            Duration::hours(SESSION_TTL_HOURS),
        );
        self.session_repo.create(&session).await?;

        Ok(AcceptOutcome {
            token: session_token,
            user_id,
            organization_id: invite.organization_id().clone(),
        })
    }
}
