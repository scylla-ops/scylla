//! The invitation's reads. One block per query, in the order it runs: the struct, its
//! permission, its output type, what `Fetch` reads.

use super::InvitationUseCases;
use crate::application::{InvitationRepository, OrganizationRepository};
use crate::domain::errors::DomainResult;
use crate::domain::ids::OrganizationId;
use crate::domain::invitation::Invitation;
use crate::domain::permission::Permission;
use async_trait::async_trait;
use scylla_auth::authz::PermissionService;
use scylla_extension::{Authorized, Describe, Fetch, Fetched, Query, Run};

#[derive(Debug)]
pub struct ListInvitations {
    pub organization_id: OrganizationId,
}

impl Describe for ListInvitations {
    fn permission(&self) -> Permission {
        Permission::ManageInvitations(self.organization_id.clone())
    }
}

impl Query for ListInvitations {
    type Output = Vec<Invitation>;
}

#[async_trait]
impl<I, O, PS> Run<Fetch<ListInvitations>> for InvitationUseCases<I, O, PS>
where
    I: InvitationRepository,
    O: OrganizationRepository + Send + Sync,
    PS: PermissionService,
{
    async fn run(
        &self,
        input: Authorized<ListInvitations>,
    ) -> DomainResult<Fetched<ListInvitations>> {
        let invitations = self
            .invite_repo
            .list_pending(&input.command().organization_id)
            .await?;
        Ok(input.fetched(invitations))
    }
}
