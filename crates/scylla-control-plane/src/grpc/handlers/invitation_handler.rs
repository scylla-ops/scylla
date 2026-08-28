use crate::application::authz::policy::PolicyControl;
use crate::application::{
    HashService, InvitationRepository, InvitationUseCases, OrganizationRepository,
    PermissionService, SessionRepository, UserRepository,
};
use crate::extract_auth_context;
use crate::grpc::convert::{optional, required, ts, wrap};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::domain::ids::{InvitationId, OrganizationId};
use scylla_core::domain::invitation::Invitation;
use scylla_core::domain::invitation::InvitationStatus as DomainInvitationStatus;
use scylla_core::domain::role::RoleName;
use scylla_core::domain::user::{Email, Password, Username};
use scylla_protocol::invitation::v1::{
    AcceptInvitationRequest, AcceptInvitationResponse, CreateInvitationRequest,
    CreateInvitationResponse, Invitation as ProtoInvitation, InvitationStatus,
    ListInvitationsRequest, ListInvitationsResponse, RevokeInvitationRequest,
    RevokeInvitationResponse, invitation_accept_service_server::InvitationAcceptService,
    invitation_service_server::InvitationService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct InvitationHandler<I, PS, O, U, H, S, PC>
where
    I: InvitationRepository,
    PS: PermissionService,
    O: OrganizationRepository,
    U: UserRepository,
    H: HashService,
    S: SessionRepository,
    PC: PolicyControl,
{
    use_cases: Arc<InvitationUseCases<I, PS, O, U, H, S, PC>>,
}

// Manual Clone: only the `Arc` is cloned, so no `Clone` bound on the generics
// (derive would wrongly require `Argon2HashService: Clone`, etc.).
impl<I, PS, O, U, H, S, PC> Clone for InvitationHandler<I, PS, O, U, H, S, PC>
where
    I: InvitationRepository,
    PS: PermissionService,
    O: OrganizationRepository,
    U: UserRepository,
    H: HashService,
    S: SessionRepository,
    PC: PolicyControl,
{
    fn clone(&self) -> Self {
        Self {
            use_cases: self.use_cases.clone(),
        }
    }
}

/// Domain status → proto enum. Total: every domain variant has a real proto
/// variant, so `Unspecified` is never produced.
fn status_to_proto(status: DomainInvitationStatus) -> InvitationStatus {
    match status {
        DomainInvitationStatus::Pending => InvitationStatus::Pending,
        DomainInvitationStatus::Accepted => InvitationStatus::Accepted,
        DomainInvitationStatus::Revoked => InvitationStatus::Revoked,
    }
}

fn to_proto(i: &Invitation) -> ProtoInvitation {
    ProtoInvitation {
        invitation_id: wrap(i.id().to_string()),
        organization_id: wrap(i.organization_id().to_string()),
        email: wrap(i.email().as_str().to_string()),
        role: i.role().and_then(|r| wrap(r.as_str().to_string())),
        status: status_to_proto(i.status()) as i32,
        expires_at: ts(i.expires_at()),
    }
}

#[async_trait::async_trait]
impl<
    I: InvitationRepository + 'static,
    PS: PermissionService + Send + Sync + 'static,
    O: OrganizationRepository + Send + Sync + 'static,
    U: UserRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    S: SessionRepository + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
> InvitationService for InvitationHandler<I, PS, O, U, H, S, PC>
{
    async fn create_invitation(
        &self,
        request: Request<CreateInvitationRequest>,
    ) -> Result<Response<CreateInvitationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(required(req.organization_id, "organization_id")?);
        let email = Email::new(&required(req.email, "email")?).map_err(domain_error_to_status)?;
        let role = optional(req.role)
            .as_deref()
            .map(RoleName::new)
            .transpose()
            .map_err(domain_error_to_status)?;

        let invite = self
            .use_cases
            .create_invite(&caller, org_id, email, role)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(CreateInvitationResponse {
            invitation: Some(to_proto(&invite)),
        }))
    }

    async fn list_invitations(
        &self,
        request: Request<ListInvitationsRequest>,
    ) -> Result<Response<ListInvitationsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(required(req.organization_id, "organization_id")?);
        let invites = self
            .use_cases
            .list_pending(&caller, &org_id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(ListInvitationsResponse {
            invitations: invites.iter().map(to_proto).collect(),
        }))
    }

    async fn revoke_invitation(
        &self,
        request: Request<RevokeInvitationRequest>,
    ) -> Result<Response<RevokeInvitationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = InvitationId::new(required(req.invitation_id, "invitation_id")?);
        self.use_cases
            .revoke(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(RevokeInvitationResponse {}))
    }
}

#[async_trait::async_trait]
impl<
    I: InvitationRepository + 'static,
    PS: PermissionService + Send + Sync + 'static,
    O: OrganizationRepository + Send + Sync + 'static,
    U: UserRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    S: SessionRepository + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
> InvitationAcceptService for InvitationHandler<I, PS, O, U, H, S, PC>
{
    async fn accept_invitation(
        &self,
        request: Request<AcceptInvitationRequest>,
    ) -> Result<Response<AcceptInvitationResponse>, Status> {
        let req = request.into_inner();
        let username = Username::new(&req.username).map_err(domain_error_to_status)?;
        let password = Password::new(&req.password).map_err(domain_error_to_status)?;

        let outcome = self
            .use_cases
            .accept(&req.token, username, password)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(AcceptInvitationResponse {
            token: outcome.token,
            user_id: wrap(outcome.user_id.to_string()),
            organization_id: wrap(outcome.organization_id.to_string()),
        }))
    }
}
