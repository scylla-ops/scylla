use crate::application::authz::grant::{
    Grant, ORGANIZATION_MEMBER_ROLE, Principal, Scope, validate_role_in_db,
};
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::role::RoleRepository;
use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::application::invitation::repository::InvitationRepository;
use crate::application::invitation::token::mint_invitation_token;
use crate::application::mail::Mailer;
use crate::application::{HashService, OrganizationRepository, SessionRepository, UserRepository};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{InvitationId, OrganizationId, UserId};
use crate::domain::invitation::Invitation;
use crate::domain::permission::Permission;
use crate::domain::role::RoleName;
use crate::domain::session::Session;
use crate::domain::user::User;
use crate::domain::user::{Email, Password, Username};
use chrono::Duration;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

const SESSION_TTL_HOURS: i64 = 24;

/// What accepting an invitation returns.
pub struct AcceptOutcome {
    pub token: String,
    pub user_id: UserId,
    pub organization_id: OrganizationId,
}

/// Member invitations. Creating an invite is gated by the same permission
/// as adding a member (`AddOrganizationMember`), so an org-admin can invite.
/// Accepting is public (the token is the credential).
///
/// The mailer is held as `Arc<dyn Mailer>` rather than a generic parameter
/// (unlike the other collaborators): the concrete transport — real SMTP via
/// `LettreMailer` or the `NoopMailer` fallback — is selected at runtime from
/// configuration, which requires dynamic dispatch.
#[derive(Constructor)]
#[allow(clippy::too_many_arguments)]
pub struct InvitationUseCases<I, PS, O, U, H, S, PC>
where
    I: InvitationRepository,
    PS: PermissionService,
    O: OrganizationRepository,
    U: UserRepository,
    H: HashService,
    S: SessionRepository,
    PC: PolicyControl,
{
    invite_repo: Arc<I>,
    permission_service: Arc<PS>,
    mailer: Arc<dyn Mailer>,
    org_repo: Arc<O>,
    user_repo: Arc<U>,
    hash_service: Arc<H>,
    session_repo: Arc<S>,
    policy_control: Arc<PC>,
    /// Role catalog, for validating an invite's role against the DB — the same
    /// check `CreateGrant` uses, so an invite can only carry a grantable role.
    role_repo: Arc<dyn RoleRepository>,
}

impl<I, PS, O, U, H, S, PC> InvitationUseCases<I, PS, O, U, H, S, PC>
where
    I: InvitationRepository,
    PS: PermissionService,
    O: OrganizationRepository,
    U: UserRepository,
    H: HashService,
    S: SessionRepository,
    PC: PolicyControl,
{
    #[instrument(skip_all, fields(org_id = %organization_id, email = %email))]
    pub async fn create_invite(
        &self,
        caller: &CallerContext,
        organization_id: OrganizationId,
        email: Email,
        role: Option<RoleName>,
    ) -> DomainResult<Invitation> {
        self.permission_service
            .check(
                caller,
                Permission::ManageInvitations(organization_id.clone()),
            )
            .await?;

        // An invite mints an Organization-scoped grant on accept; reject a role
        // that isn't assignable on an org now, before persisting/emailing it.
        if let Some(role) = &role {
            validate_role_in_db(
                &*self.role_repo,
                role,
                &Scope::Organization(organization_id.clone()),
            )
            .await?;
        }

        let org = self.org_repo.find_by_id(&organization_id).await?;
        let invited_by = match caller {
            CallerContext::User(id) => id.clone(),
            _ => UserId::new("system"),
        };
        let invite = Invitation::create(
            organization_id,
            email,
            role,
            invited_by,
            mint_invitation_token(),
        );
        self.invite_repo.create(&invite).await?;

        let body = format!(
            "<p>You've been invited to join <b>{}</b> on Scylla.</p>\
             <p>Use this token to accept: <code>{}</code></p>",
            org.name().as_str(),
            invite.token()
        );
        // Email delivery is best-effort: a transient SMTP failure must not lose
        // the persisted invitation (it can be re-sent).
        if let Err(e) = self
            .mailer
            .send(invite.email(), "You've been invited to Scylla", &body)
            .await
        {
            tracing::warn!(error = %e, invite_id = %invite.id(), "invite email send failed");
        }

        Ok(invite)
    }

    #[instrument(skip_all, fields(org_id = %organization_id))]
    pub async fn list_pending(
        &self,
        caller: &CallerContext,
        organization_id: &OrganizationId,
    ) -> DomainResult<Vec<Invitation>> {
        self.permission_service
            .check(
                caller,
                Permission::ManageInvitations(organization_id.clone()),
            )
            .await?;
        self.invite_repo.list_pending(organization_id).await
    }

    #[instrument(skip_all, fields(invite_id = %invite_id))]
    pub async fn revoke(
        &self,
        caller: &CallerContext,
        invite_id: &InvitationId,
    ) -> DomainResult<()> {
        // Resolve the invite's org to scope the permission check to it.
        let invite = self.invite_repo.find_by_id(invite_id).await?;
        self.permission_service
            .check(
                caller,
                Permission::ManageInvitations(invite.organization_id().clone()),
            )
            .await?;
        self.invite_repo.revoke(invite_id).await
    }

    /// Public accept: the token is the credential. Creates the user if their
    /// email is new, adds them to the org, mints the optional role grant, and
    /// issues a session — atomically.
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

        // Existing account with this email joins directly; otherwise create one.
        let (new_user, user_id) =
            if let Ok(existing) = self.user_repo.find_by_email(invite.email()).await {
                (None, existing.id().clone())
            } else {
                let password_hash = self.hash_service.hash(&password).await?;
                let user = User::create(username, Some(invite.email().clone()), password_hash);
                let id = user.id().clone();
                (Some(user), id)
            };

        // The grant is what joins them to the organization, so an invitation
        // without a named role still mints one: `organization-member`, which
        // confers only the ability to see that the organization exists. Without
        // it the invitee would accept and land on an empty account.
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
