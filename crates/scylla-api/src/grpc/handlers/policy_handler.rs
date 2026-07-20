use crate::extract_auth_context;
use crate::grpc::convert::{permission_from_key, required, scope_kind_to_proto, ts, wrap};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    PermissionService, PolicyControl, PolicyDefinition, PolicyRepository, PolicyUseCases,
    resource_home_scope,
};
use scylla_core::domain::entities::CedarPolicyId;
use scylla_core::domain::errors::DomainError;
use scylla_protocol::authz::v1::{
    AuthzAction, CreatePolicyRequest, CreatePolicyResponse, DeletePolicyRequest,
    DeletePolicyResponse, ListAuthzVocabularyRequest, ListAuthzVocabularyResponse,
    ListPoliciesRequest, ListPoliciesResponse, Policy, SetPolicyEnabledRequest,
    SetPolicyEnabledResponse, UpdatePolicyRequest, UpdatePolicyResponse, ValidatePolicyRequest,
    ValidatePolicyResponse, policy_service_server::PolicyService, validate_policy_response,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct PolicyHandler<R: PolicyRepository, PC: PolicyControl, PS: PermissionService> {
    use_cases: Arc<PolicyUseCases<R, PC, PS>>,
}

#[async_trait::async_trait]
impl<
    R: PolicyRepository + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> PolicyService for PolicyHandler<R, PC, PS>
{
    async fn create_policy(
        &self,
        request: Request<CreatePolicyRequest>,
    ) -> Result<Response<CreatePolicyResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();

        let policy = self
            .use_cases
            .create(&caller, req.description, req.text)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(CreatePolicyResponse {
            policy: Some(policy_to_proto(&policy)),
        }))
    }

    async fn update_policy(
        &self,
        request: Request<UpdatePolicyRequest>,
    ) -> Result<Response<UpdatePolicyResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = CedarPolicyId::new(&required(req.policy_id, "policy_id")?);

        let policy = self
            .use_cases
            .update(&caller, &id, req.description, req.text)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(UpdatePolicyResponse {
            policy: Some(policy_to_proto(&policy)),
        }))
    }

    async fn set_policy_enabled(
        &self,
        request: Request<SetPolicyEnabledRequest>,
    ) -> Result<Response<SetPolicyEnabledResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = CedarPolicyId::new(&required(req.policy_id, "policy_id")?);

        let policy = self
            .use_cases
            .set_enabled(&caller, &id, req.enabled)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(SetPolicyEnabledResponse {
            policy: Some(policy_to_proto(&policy)),
        }))
    }

    async fn delete_policy(
        &self,
        request: Request<DeletePolicyRequest>,
    ) -> Result<Response<DeletePolicyResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = CedarPolicyId::new(&required(req.policy_id, "policy_id")?);

        self.use_cases
            .delete(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeletePolicyResponse {}))
    }

    async fn list_policies(
        &self,
        request: Request<ListPoliciesRequest>,
    ) -> Result<Response<ListPoliciesResponse>, Status> {
        let caller = caller!(request);

        let policies = self
            .use_cases
            .list(&caller)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ListPoliciesResponse {
            policies: policies.iter().map(policy_to_proto).collect(),
        }))
    }

    async fn validate_policy(
        &self,
        request: Request<ValidatePolicyRequest>,
    ) -> Result<Response<ValidatePolicyResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();

        // A validation failure is a successful RPC reporting the `invalid` arm;
        // only auth/infra errors surface as a gRPC error status.
        let result = match self.use_cases.validate(&caller, &req.text).await {
            Ok(()) => validate_policy_response::Result::Valid(validate_policy_response::Valid {}),
            Err(DomainError::Validation(error)) => {
                validate_policy_response::Result::Invalid(validate_policy_response::Invalid {
                    error,
                })
            }
            Err(e) => return Err(domain_error_to_status(e)),
        };

        Ok(Response::new(ValidatePolicyResponse {
            result: Some(result),
        }))
    }

    async fn list_authz_vocabulary(
        &self,
        request: Request<ListAuthzVocabularyRequest>,
    ) -> Result<Response<ListAuthzVocabularyResponse>, Status> {
        let caller = caller!(request);

        let actions = self
            .use_cases
            .authz_vocabulary(&caller)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ListAuthzVocabularyResponse {
            // resource_type is derivable from the permission, so it never ships;
            // we keep min_scope (the one derived fact the client consumes) and
            // compute it server-side from the permission's resource type.
            actions: actions
                .iter()
                .map(|(key, resource_type)| AuthzAction {
                    permission: permission_from_key(key).map_or(0, |p| p as i32),
                    min_scope: scope_kind_to_proto(resource_home_scope(resource_type)) as i32,
                })
                .collect(),
        }))
    }
}

fn policy_to_proto(p: &PolicyDefinition) -> Policy {
    Policy {
        policy_id: wrap(p.id.to_string()),
        description: p.description.clone(),
        text: p.text.clone(),
        enabled: p.enabled,
        created_by: p.created_by.clone(),
        created_at: ts(p.created_at),
        updated_at: ts(p.updated_at),
    }
}
