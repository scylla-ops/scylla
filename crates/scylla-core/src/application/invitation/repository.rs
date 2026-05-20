use crate::application::permission::grant::Grant;
use crate::domain::entities::{Invitation, InvitationId, OrganizationId, User, UserId};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence for invitations. `accept_atomic` performs the join (optionally
/// creating the user and a scoped grant) in a single transaction.
#[async_trait]
pub trait InvitationRepository: Send + Sync {
    async fn create(&self, invite: &Invitation) -> DomainResult<()>;
    async fn find_by_id(&self, id: &InvitationId) -> DomainResult<Invitation>;
    async fn find_by_token(&self, token: &str) -> DomainResult<Invitation>;
    async fn list_pending(&self, org_id: &OrganizationId) -> DomainResult<Vec<Invitation>>;
    async fn revoke(&self, id: &InvitationId) -> DomainResult<()>;
    /// Atomic accept: insert `new_user` if Some, add the member to the org,
    /// insert `grant` if Some, and mark the invitation accepted — all or nothing.
    async fn accept_atomic(
        &self,
        invite_id: &InvitationId,
        new_user: Option<&User>,
        member: &UserId,
        organization_id: &OrganizationId,
        grant: Option<&Grant>,
    ) -> DomainResult<()>;
}
