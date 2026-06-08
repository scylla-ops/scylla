use crate::extract_auth_context;
use crate::grpc::convert::{required, ts, wrap};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::authz::policy::PolicyControl;
use scylla_core::application::{
    HashService, InvitationRepository, InvitationUseCases, OrganizationRepository,
    PermissionService, SessionRepository, UserRepository,
};
use scylla_core::domain::entities::{Invitation, InvitationId, OrganizationId};
use scylla_core::domain::value_objects::role::name::RoleName;
use scylla_core::domain::value_objects::user::{Email, Password, Username};
use scylla_protocol::services::invitation::{
    AcceptInviteRequest, AcceptInviteResponse, CreateInviteRequest, InvitationResponse,
    ListInvitesRequest, ListInvitesResponse, RevokeInviteRequest, RevokeInviteResponse,
    invitation_accept_service_server::InvitationAcceptService,
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

fn to_proto(i: &Invitation) -> InvitationResponse {
    InvitationResponse {
        id: wrap(i.id().to_string()),
        organization_id: wrap(i.organization_id().to_string()),
        email: wrap(i.email().as_str().to_string()),
        role: i.role().map(|r| r.as_str().to_string()),
        status: i.status().as_str().to_string(),
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
    async fn create_invite(
        &self,
        request: Request<CreateInviteRequest>,
    ) -> Result<Response<InvitationResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(required(req.organization_id, "organization_id")?);
        let email = Email::new(&required(req.email, "email")?).map_err(domain_error_to_status)?;
        let role = req
            .role
            .as_deref()
            .map(RoleName::new)
            .transpose()
            .map_err(domain_error_to_status)?;

        let invite = self
            .use_cases
            .create_invite(&caller, org_id, email, role)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(to_proto(&invite)))
    }

    async fn list_invites(
        &self,
        request: Request<ListInvitesRequest>,
    ) -> Result<Response<ListInvitesResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let org_id = OrganizationId::new(required(req.organization_id, "organization_id")?);
        let invites = self
            .use_cases
            .list_pending(&caller, &org_id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(ListInvitesResponse {
            invitations: invites.iter().map(to_proto).collect(),
        }))
    }

    async fn revoke_invite(
        &self,
        request: Request<RevokeInviteRequest>,
    ) -> Result<Response<RevokeInviteResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = InvitationId::new(required(req.invitation_id, "invitation_id")?);
        self.use_cases
            .revoke(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(RevokeInviteResponse {}))
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
    async fn accept_invite(
        &self,
        request: Request<AcceptInviteRequest>,
    ) -> Result<Response<AcceptInviteResponse>, Status> {
        let req = request.into_inner();
        let username = Username::new(&req.username).map_err(domain_error_to_status)?;
        let password = Password::new(&req.password).map_err(domain_error_to_status)?;

        let outcome = self
            .use_cases
            .accept(&req.token, username, password)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(AcceptInviteResponse {
            token: outcome.token,
            user_id: wrap(outcome.user_id.to_string()),
            organization_id: wrap(outcome.organization_id.to_string()),
        }))
    }
}
