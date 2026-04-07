use crate::extract_auth_context;
use crate::grpc::mappers::domain_error_to_status;
use crate::grpc::mappers::permission_mapper::{
    domain_act_to_proto, domain_resource_to_proto, domain_scope_to_proto, proto_act_to_domain,
    proto_resource_to_domain, proto_scope_to_domain,
};
use derive_more::Constructor;
use protocol::services::permission::{
    Act as ProtoAct, AddGroupingPolicyRequest, AddGroupingPolicyResponse, AddPolicyRequest,
    AddPolicyResponse, GroupingPolicyEntry, ListGroupingPoliciesRequest,
    ListGroupingPoliciesResponse, ListPoliciesRequest, ListPoliciesResponse, PolicyEntry,
    RemoveGroupingPolicyRequest, RemoveGroupingPolicyResponse, RemovePolicyRequest,
    RemovePolicyResponse, permission_service_server::PermissionService as PermissionServiceTrait,
};
use scylla_core::application::PermissionUseCases;
use scylla_core::application::ports::services::permission_service::PermissionService;
use scylla_core::domain::entities::UserId;
use scylla_core::domain::value_objects::permission::policy::{self, GroupingPolicy, Policy};
use scylla_core::domain::value_objects::role::name::RoleName;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_interceptor::AuthContext;
    use async_trait::async_trait;
    use protocol::services::permission::permission_service_server::PermissionService as PermissionServiceTrait;
    use protocol::services::permission::{Resource, ResourceType, Scope, ScopeType};
    use scylla_core::application::PermissionUseCases;
    use scylla_core::application::ports::services::permission_service::PermissionService;
    use scylla_core::domain::entities::EntityId;
    use scylla_core::domain::errors::DomainResult;
    use scylla_core::domain::value_objects::permission::policy::{GroupingPolicy, Policy};
    use std::sync::Arc;

    // ── Stubs ──────────────────────────────────────────────────

    struct StubPermission {
        check_fn: Option<Box<dyn Fn() -> DomainResult<bool> + Send + Sync>>,
        add_policy_fn: Option<Box<dyn Fn() -> DomainResult<bool> + Send + Sync>>,
        remove_policy_fn: Option<Box<dyn Fn() -> DomainResult<bool> + Send + Sync>>,
        list_policies_fn:
            Option<Box<dyn Fn(Option<&str>) -> DomainResult<Vec<(String, Policy)>> + Send + Sync>>,
        add_grouping_fn: Option<Box<dyn Fn() -> DomainResult<bool> + Send + Sync>>,
        remove_grouping_fn: Option<Box<dyn Fn() -> DomainResult<bool> + Send + Sync>>,
        list_grouping_fn: Option<
            Box<dyn Fn(Option<&str>) -> DomainResult<Vec<(String, GroupingPolicy)>> + Send + Sync>,
        >,
    }

    impl Default for StubPermission {
        fn default() -> Self {
            Self {
                check_fn: Some(Box::new(|| Ok(true))),
                add_policy_fn: Some(Box::new(|| Ok(true))),
                remove_policy_fn: Some(Box::new(|| Ok(true))),
                list_policies_fn: Some(Box::new(|_| Ok(vec![]))),
                add_grouping_fn: Some(Box::new(|| Ok(true))),
                remove_grouping_fn: Some(Box::new(|| Ok(true))),
                list_grouping_fn: Some(Box::new(|_| Ok(vec![]))),
            }
        }
    }

    #[async_trait]
    impl PermissionService for StubPermission {
        async fn check(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> {
            (self.check_fn.as_ref().unwrap())()
        }
        async fn add_policy(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> {
            (self.add_policy_fn.as_ref().unwrap())()
        }
        async fn remove_policy(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> {
            (self.remove_policy_fn.as_ref().unwrap())()
        }
        async fn list_policies(&self, s: Option<&str>) -> DomainResult<Vec<(String, Policy)>> {
            (self.list_policies_fn.as_ref().unwrap())(s)
        }
        async fn add_grouping_policy(
            &self,
            _s: impl EntityId,
            _p: GroupingPolicy,
        ) -> DomainResult<bool> {
            (self.add_grouping_fn.as_ref().unwrap())()
        }
        async fn remove_grouping_policy(
            &self,
            _s: impl EntityId,
            _p: GroupingPolicy,
        ) -> DomainResult<bool> {
            (self.remove_grouping_fn.as_ref().unwrap())()
        }
        async fn list_grouping_policies(
            &self,
            s: Option<&str>,
        ) -> DomainResult<Vec<(String, GroupingPolicy)>> {
            (self.list_grouping_fn.as_ref().unwrap())(s)
        }
    }

    // ── Helpers ─────────────────────────────────────────────────

    fn authed_request<T>(body: T) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut()
            .insert(AuthContext::new(UserId::generate()));
        req
    }

    fn make_handler(stub: StubPermission) -> PermissionHandler<StubPermission> {
        let stub = Arc::new(stub);
        let uc = Arc::new(PermissionUseCases::new(stub.clone()));
        PermissionHandler::new(uc, stub)
    }

    // ── Tests ───────────────────────────────────────────────────

    #[tokio::test]
    async fn add_policy_success() {
        let handler = make_handler(StubPermission::default());
        let req = authed_request(AddPolicyRequest {
            subject: "user123".into(),
            scope: Some(Scope {
                r#type: ScopeType::ScopeSystem.into(),
                id: None,
            }),
            resource: Some(Resource {
                r#type: ResourceType::ResourceUser.into(),
                id: None,
            }),
            act: ProtoAct::Read.into(),
        });

        let resp = handler.add_policy(req).await.unwrap();
        assert!(resp.into_inner().added);
    }

    #[tokio::test]
    async fn remove_policy_success() {
        let handler = make_handler(StubPermission::default());
        let req = authed_request(RemovePolicyRequest {
            subject: "user123".into(),
            scope: Some(Scope {
                r#type: ScopeType::ScopeSystem.into(),
                id: None,
            }),
            resource: Some(Resource {
                r#type: ResourceType::ResourceUser.into(),
                id: None,
            }),
            act: ProtoAct::Read.into(),
        });

        let resp = handler.remove_policy(req).await.unwrap();
        assert!(resp.into_inner().removed);
    }

    #[tokio::test]
    async fn list_policies_returns_empty() {
        let handler = make_handler(StubPermission::default());
        let req = authed_request(ListPoliciesRequest { subject: None });

        let resp = handler.list_policies(req).await.unwrap();
        assert!(resp.into_inner().policies.is_empty());
    }

    #[tokio::test]
    async fn add_grouping_policy_success() {
        let handler = make_handler(StubPermission::default());
        let req = authed_request(AddGroupingPolicyRequest {
            subject: "user123".into(),
            role: "admin".into(),
            scope: Some(Scope {
                r#type: ScopeType::ScopeSystem.into(),
                id: None,
            }),
        });

        let resp = handler.add_grouping_policy(req).await.unwrap();
        assert!(resp.into_inner().added);
    }

    #[tokio::test]
    async fn remove_grouping_policy_success() {
        let handler = make_handler(StubPermission::default());
        let req = authed_request(RemoveGroupingPolicyRequest {
            subject: "user123".into(),
            role: "admin".into(),
            scope: Some(Scope {
                r#type: ScopeType::ScopeSystem.into(),
                id: None,
            }),
        });

        let resp = handler.remove_grouping_policy(req).await.unwrap();
        assert!(resp.into_inner().removed);
    }

    #[tokio::test]
    async fn list_grouping_policies_returns_empty() {
        let handler = make_handler(StubPermission::default());
        let req = authed_request(ListGroupingPoliciesRequest { subject: None });

        let resp = handler.list_grouping_policies(req).await.unwrap();
        assert!(resp.into_inner().grouping_policies.is_empty());
    }

    #[tokio::test]
    async fn add_policy_without_auth_fails() {
        let handler = make_handler(StubPermission::default());
        let req = Request::new(AddPolicyRequest {
            subject: "user123".into(),
            scope: Some(Scope {
                r#type: ScopeType::ScopeSystem.into(),
                id: None,
            }),
            resource: Some(Resource {
                r#type: ResourceType::ResourceUser.into(),
                id: None,
            }),
            act: ProtoAct::Read.into(),
        });

        let err = handler.add_policy(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Internal);
    }

    #[tokio::test]
    async fn add_policy_missing_scope_fails() {
        let handler = make_handler(StubPermission::default());
        let req = authed_request(AddPolicyRequest {
            subject: "user123".into(),
            scope: None,
            resource: Some(Resource {
                r#type: ResourceType::ResourceUser.into(),
                id: None,
            }),
            act: ProtoAct::Read.into(),
        });

        let err = handler.add_policy(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }
}
