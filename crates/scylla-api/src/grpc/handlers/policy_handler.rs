use crate::extract_auth_context;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    PermissionService, PolicyControl, PolicyDefinition, PolicyRepository, PolicyUseCases,
};
use scylla_core::domain::entities::CedarPolicyId;
use scylla_core::domain::errors::DomainError;
use scylla_protocol::services::permission::{
    CreatePolicyRequest, DeletePolicyRequest, DeletePolicyResponse, ListPoliciesRequest,
    ListPoliciesResponse, Policy, PolicyResponse, SetPolicyEnabledRequest, UpdatePolicyRequest,
    ValidatePolicyRequest, ValidatePolicyResponse, policy_service_server::PolicyService,
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
    ) -> Result<Response<PolicyResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();

        let policy = self
            .use_cases
            .create(&caller, req.description, req.text)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(PolicyResponse {
            policy: Some(policy_to_proto(&policy)),
        }))
    }

    async fn update_policy(
        &self,
        request: Request<UpdatePolicyRequest>,
    ) -> Result<Response<PolicyResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = CedarPolicyId::new(&req.id);

        let policy = self
            .use_cases
            .update(&caller, &id, req.description, req.text)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(PolicyResponse {
            policy: Some(policy_to_proto(&policy)),
        }))
    }

    async fn set_policy_enabled(
        &self,
        request: Request<SetPolicyEnabledRequest>,
    ) -> Result<Response<PolicyResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = CedarPolicyId::new(&req.id);

        let policy = self
            .use_cases
            .set_enabled(&caller, &id, req.enabled)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(PolicyResponse {
            policy: Some(policy_to_proto(&policy)),
        }))
    }

    async fn delete_policy(
        &self,
        request: Request<DeletePolicyRequest>,
    ) -> Result<Response<DeletePolicyResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = CedarPolicyId::new(&req.id);

        self.use_cases
            .delete(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeletePolicyResponse { deleted: true }))
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

        // A validation failure is a successful RPC reporting `valid = false`;
        // only auth/infra errors surface as a gRPC error status.
        match self.use_cases.validate(&caller, &req.text).await {
            Ok(()) => Ok(Response::new(ValidatePolicyResponse {
                valid: true,
                error: None,
            })),
            Err(DomainError::Validation(message)) => Ok(Response::new(ValidatePolicyResponse {
                valid: false,
                error: Some(message),
            })),
            Err(e) => Err(domain_error_to_status(e)),
        }
    }
}

fn policy_to_proto(p: &PolicyDefinition) -> Policy {
    Policy {
        id: p.id.to_string(),
        description: p.description.clone(),
        text: p.text.clone(),
        enabled: p.enabled,
        created_by: p.created_by.clone(),
        created_at: p.created_at.to_rfc3339(),
        updated_at: p.updated_at.to_rfc3339(),
    }
}
