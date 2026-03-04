use crate::extract_auth_context;
use crate::grpc::mappers::domain_error_to_status;
use crate::grpc::mappers::permission_mapper::{
    domain_act_to_proto, domain_resource_to_proto, domain_scope_to_proto, proto_act_to_domain,
    proto_resource_to_domain, proto_scope_to_domain,
};
use application::PermissionUseCases;
use derive_more::Constructor;
use domain::entities::UserId;
use domain::ports::services::permission_service::PermissionService;
use domain::value_objects::permission::policy::{self, GroupingPolicy, Policy};
use domain::value_objects::role::name::RoleName;
use protocol::services::permission::{
    Act as ProtoAct, AddGroupingPolicyRequest, AddGroupingPolicyResponse, AddPolicyRequest,
    AddPolicyResponse, GroupingPolicyEntry, ListGroupingPoliciesRequest,
    ListGroupingPoliciesResponse, ListPoliciesRequest, ListPoliciesResponse, PolicyEntry,
    RemoveGroupingPolicyRequest, RemoveGroupingPolicyResponse, RemovePolicyRequest,
    RemovePolicyResponse, permission_service_server::PermissionService as PermissionServiceTrait,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct PermissionHandler<PS: PermissionService> {
    use_cases: Arc<PermissionUseCases<PS>>,
    permission_checker: Arc<PS>,
}

/// Build a domain `Policy` from strongly-typed proto values.
fn proto_to_policy(
    scope: protocol::services::permission::Scope,
    resource: protocol::services::permission::Resource,
    act_i32: i32,
) -> Result<Policy, Status> {
    let act = proto_act_to_domain(
        ProtoAct::try_from(act_i32)
            .map_err(|_| Status::invalid_argument(format!("Invalid act value: {}", act_i32)))?,
    )
    .map_err(domain_error_to_status)?;

    let scope = proto_scope_to_domain(scope).map_err(domain_error_to_status)?;
    let resource = proto_resource_to_domain(resource).map_err(domain_error_to_status)?;
    Ok(Policy::new(scope, resource, act))
}

/// Convert a typed policy row to a proto `PolicyEntry`.
fn row_to_policy_entry(subject: String, policy: Policy) -> PolicyEntry {
    PolicyEntry {
        subject,
        scope: Some(domain_scope_to_proto(&policy.scope)),
        resource: Some(domain_resource_to_proto(&policy.resource)),
        act: domain_act_to_proto(&policy.act).into(),
    }
}

/// Convert a typed grouping-policy row to a proto entry.
fn row_to_grouping_entry(subject: String, policy: GroupingPolicy) -> GroupingPolicyEntry {
    GroupingPolicyEntry {
        subject,
        role: policy.role.into_string(),
        scope: Some(domain_scope_to_proto(&policy.scope)),
    }
}

#[async_trait::async_trait]
impl<PS: PermissionService + Send + Sync + 'static> PermissionServiceTrait
    for PermissionHandler<PS>
{
    async fn add_policy(
        &self,
        request: Request<AddPolicyRequest>,
    ) -> Result<Response<AddPolicyResponse>, Status> {
        require_permission!(self, request, policy::permission::manage());
        let req = request.into_inner();
        let subject = UserId::new(req.subject);
        let p = proto_to_policy(
            req.scope
                .ok_or_else(|| Status::invalid_argument("Missing scope"))?,
            req.resource
                .ok_or_else(|| Status::invalid_argument("Missing resource"))?,
            req.act,
        )?;

        let added = self
            .use_cases
            .add_policy(subject, p)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(AddPolicyResponse { added }))
    }

    async fn remove_policy(
        &self,
        request: Request<RemovePolicyRequest>,
    ) -> Result<Response<RemovePolicyResponse>, Status> {
        require_permission!(self, request, policy::permission::manage());
        let req = request.into_inner();
        let subject = UserId::new(req.subject);
        let p = proto_to_policy(
            req.scope
                .ok_or_else(|| Status::invalid_argument("Missing scope"))?,
            req.resource
                .ok_or_else(|| Status::invalid_argument("Missing resource"))?,
            req.act,
        )?;

        let removed = self
            .use_cases
            .remove_policy(subject, p)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RemovePolicyResponse { removed }))
    }

    async fn list_policies(
        &self,
        request: Request<ListPoliciesRequest>,
    ) -> Result<Response<ListPoliciesResponse>, Status> {
        require_permission!(self, request, policy::permission::list());
        let req = request.into_inner();

        let rows = self
            .use_cases
            .list_policies(req.subject.as_deref())
            .await
            .map_err(domain_error_to_status)?;

        let policies = rows
            .into_iter()
            .map(|(sub, policy)| row_to_policy_entry(sub, policy))
            .collect();

        Ok(Response::new(ListPoliciesResponse { policies }))
    }

    async fn add_grouping_policy(
        &self,
        request: Request<AddGroupingPolicyRequest>,
    ) -> Result<Response<AddGroupingPolicyResponse>, Status> {
        require_permission!(self, request, policy::permission::manage());
        let req = request.into_inner();
        let subject = UserId::new(req.subject);
        let scope_proto = req
            .scope
            .ok_or_else(|| Status::invalid_argument("Missing scope"))?;
        let scope = proto_scope_to_domain(scope_proto).map_err(domain_error_to_status)?;
        let role = RoleName::new(req.role).map_err(domain_error_to_status)?;
        let gp = GroupingPolicy::new(role, scope);

        let added = self
            .use_cases
            .add_grouping_policy(subject, gp)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(AddGroupingPolicyResponse { added }))
    }

    async fn remove_grouping_policy(
        &self,
        request: Request<RemoveGroupingPolicyRequest>,
    ) -> Result<Response<RemoveGroupingPolicyResponse>, Status> {
        require_permission!(self, request, policy::permission::manage());
        let req = request.into_inner();
        let subject = UserId::new(req.subject);
        let scope_proto = req
            .scope
            .ok_or_else(|| Status::invalid_argument("Missing scope"))?;
        let scope = proto_scope_to_domain(scope_proto).map_err(domain_error_to_status)?;
        let role = RoleName::new(req.role).map_err(domain_error_to_status)?;
        let gp = GroupingPolicy::new(role, scope);

        let removed = self
            .use_cases
            .remove_grouping_policy(subject, gp)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RemoveGroupingPolicyResponse { removed }))
    }

    async fn list_grouping_policies(
        &self,
        request: Request<ListGroupingPoliciesRequest>,
    ) -> Result<Response<ListGroupingPoliciesResponse>, Status> {
        require_permission!(self, request, policy::permission::list());
        let req = request.into_inner();

        let rows = self
            .use_cases
            .list_grouping_policies(req.subject.as_deref())
            .await
            .map_err(domain_error_to_status)?;

        let grouping_policies = rows
            .into_iter()
            .map(|(sub, policy)| row_to_grouping_entry(sub, policy))
            .collect();

        Ok(Response::new(ListGroupingPoliciesResponse {
            grouping_policies,
        }))
    }
}
