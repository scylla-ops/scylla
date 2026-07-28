use crate::application::authz::grant::Grant;
use crate::domain::entities::{Invitation, InvitationId, OrganizationId, User, UserId};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence for invitations. `accept_atomic` performs the join (optionally
/// creating the user, always writing the grant that joins them) in a single
/// transaction.
#[async_trait]
pub trait InvitationRepository: Send + Sync {
    async fn create(&self, invite: &Invitation) -> DomainResult<()>;
    async fn find_by_id(&self, id: &InvitationId) -> DomainResult<Invitation>;
    async fn find_by_token(&self, token: &str) -> DomainResult<Invitation>;
    async fn list_pending(&self, org_id: &OrganizationId) -> DomainResult<Vec<Invitation>>;
    async fn revoke(&self, id: &InvitationId) -> DomainResult<()>;
    /// Atomic accept: insert `new_user` if Some, write the grant that puts them
    /// in the organization, and mark the invitation accepted — all or nothing.
    /// The grant is not optional: it *is* the join, so an accept that wrote no
    /// grant would leave the invitee unable to see the organization at all.
    async fn accept_atomic(
        &self,
        invite_id: &InvitationId,
        new_user: Option<&User>,
        member: &UserId,
        organization_id: &OrganizationId,
        grant: &Grant,
    ) -> DomainResult<()>;
}
