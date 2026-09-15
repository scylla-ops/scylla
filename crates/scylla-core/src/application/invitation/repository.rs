use crate::domain::errors::DomainResult;
use crate::domain::ids::{InvitationId, OrganizationId, UserId};
use crate::domain::invitation::Invitation;
use crate::domain::user::User;
use async_trait::async_trait;
use scylla_auth::authz::Grant;

#[async_trait]
pub trait InvitationRepository: Send + Sync {
    async fn create(&self, invite: &Invitation) -> DomainResult<()>;
    async fn find_by_id(&self, id: &InvitationId) -> DomainResult<Invitation>;
    async fn find_by_token(&self, token: &str) -> DomainResult<Invitation>;
    async fn list_pending(&self, org_id: &OrganizationId) -> DomainResult<Vec<Invitation>>;
    async fn revoke(&self, id: &InvitationId) -> DomainResult<()>;
    /// The grant is the join: an accept that wrote none would leave the invitee unable to see the org.
    async fn accept_atomic(
        &self,
        invite_id: &InvitationId,
        new_user: Option<&User>,
        member: &UserId,
        grant: &Grant,
    ) -> DomainResult<()>;
}
