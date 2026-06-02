use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::domain::clock;
use crate::domain::entities::CedarPolicyId;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::{PERMISSION_CATALOG, Permission, RESOURCE_TYPES};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

/// A runtime-managed Cedar policy. The `text` is opaque Cedar source — the
/// domain never parses it; validation and parsing happen in the infra adapter
/// behind the [`PolicyControl`] port, keeping Cedar out of the domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDefinition {
    pub id: CedarPolicyId,
    pub description: String,
    pub text: String,
    pub enabled: bool,
    /// User id or service name that created the policy (audit/forensics).
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Persistence for runtime Cedar policies (`cedar_policies` table). `list_enabled`
/// feeds the live `PolicySet` rebuild; the rest back the admin CRUD use cases.
#[async_trait]
pub trait PolicyRepository: Send + Sync {
    async fn list_enabled(&self) -> DomainResult<Vec<PolicyDefinition>>;
    async fn list_all(&self) -> DomainResult<Vec<PolicyDefinition>>;
    async fn get(&self, id: &CedarPolicyId) -> DomainResult<PolicyDefinition>;
    async fn create(&self, policy: &PolicyDefinition) -> DomainResult<()>;
    async fn update(&self, policy: &PolicyDefinition) -> DomainResult<()>;
    async fn delete(&self, id: &CedarPolicyId) -> DomainResult<()>;
}

/// Control surface over the live authorization policy set. Implemented by the
/// Cedar adapter; the application uses it to validate a candidate policy before
/// persisting (validate-on-write) and to atomically rebuild the live set after a
/// policy or grant change (hot-reload). Kept Cedar-free so it lives in the
/// application layer.
#[async_trait]
pub trait PolicyControl: Send + Sync {
    /// Parse + typecheck a candidate policy against the schema. Errors carry the
    /// engine's diagnostics so an admin can fix the policy. Rejects non-permit
    /// effects (runtime policies are permit-only).
    async fn validate_policy(&self, text: &str) -> DomainResult<()>;
    /// Rebuild the live policy set from the stores (static base + grants + DB
    /// policies) and swap it in atomically. On failure the previous set is kept.
    async fn reload(&self) -> DomainResult<()>;
}

/// Admin-only management of runtime Cedar policies. Every method is gated by
/// [`Permission::ManagePolicies`] (admin / service in practice). Writes are
/// validated before persistence and applied live via [`PolicyControl::reload`];
/// a failed reload is compensated so the store and the live set stay in sync.
#[derive(Constructor)]
pub struct PolicyUseCases<R: PolicyRepository, PC: PolicyControl, PS: PermissionService> {
    policy_repo: Arc<R>,
    policy_control: Arc<PC>,
    permission_service: Arc<PS>,
}

impl<R: PolicyRepository, PC: PolicyControl, PS: PermissionService> PolicyUseCases<R, PC, PS> {
    #[instrument(skip(self, caller))]
    pub async fn list(&self, caller: &CallerContext) -> DomainResult<Vec<PolicyDefinition>> {
        self.permission_service
            .check(caller, Permission::ManagePolicies)
            .await?;
        self.policy_repo.list_all().await
    }

    /// The authorization vocabulary a policy may reference: every action id with
    /// the resource type it targets, plus the set of resource types. Static —
    /// compiled into the binary, no DB. Gated by `ManagePolicies` like the rest
    /// of policy administration.
    #[instrument(skip(self, caller))]
    pub async fn authz_vocabulary(
        &self,
        caller: &CallerContext,
    ) -> DomainResult<(
        &'static [(&'static str, &'static str)],
        &'static [&'static str],
    )> {
        self.permission_service
            .check(caller, Permission::ManagePolicies)
            .await?;
        Ok((PERMISSION_CATALOG, RESOURCE_TYPES))
    }

    /// Dry-run validation without persisting — backs a "check before save" UX.
    #[instrument(skip(self, caller, text))]
    pub async fn validate(&self, caller: &CallerContext, text: &str) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::ManagePolicies)
            .await?;
        self.policy_control.validate_policy(text).await
    }

    #[instrument(skip(self, caller, text))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        description: String,
        text: String,
    ) -> DomainResult<PolicyDefinition> {
        self.permission_service
            .check(caller, Permission::ManagePolicies)
            .await?;
        self.policy_control.validate_policy(&text).await?;

        let now = clock::now();
        let policy = PolicyDefinition {
            id: CedarPolicyId::generate(),
            description,
            text,
            enabled: true,
            created_by: caller_id(caller),
            created_at: now,
            updated_at: now,
        };
        self.policy_repo.create(&policy).await?;

        // Apply live; on failure roll back the row so the store matches the set.
        if let Err(e) = self.policy_control.reload().await {
            let _ = self.policy_repo.delete(&policy.id).await;
            return Err(e);
        }
        Ok(policy)
    }

    #[instrument(skip(self, caller, text))]
    pub async fn update(
        &self,
        caller: &CallerContext,
        id: &CedarPolicyId,
        description: Option<String>,
        text: Option<String>,
    ) -> DomainResult<PolicyDefinition> {
        self.permission_service
            .check(caller, Permission::ManagePolicies)
            .await?;

        let previous = self.policy_repo.get(id).await?;
        let mut updated = previous.clone();
        if let Some(description) = description {
            updated.description = description;
        }
        if let Some(text) = text {
            self.policy_control.validate_policy(&text).await?;
            updated.text = text;
        }
        updated.updated_at = clock::now();
        self.policy_repo.update(&updated).await?;

        if let Err(e) = self.policy_control.reload().await {
            let _ = self.policy_repo.update(&previous).await;
            return Err(e);
        }
        Ok(updated)
    }

    #[instrument(skip(self, caller))]
    pub async fn set_enabled(
        &self,
        caller: &CallerContext,
        id: &CedarPolicyId,
        enabled: bool,
    ) -> DomainResult<PolicyDefinition> {
        self.permission_service
            .check(caller, Permission::ManagePolicies)
            .await?;

        let previous = self.policy_repo.get(id).await?;
        let mut updated = previous.clone();
        updated.enabled = enabled;
        updated.updated_at = clock::now();
        self.policy_repo.update(&updated).await?;

        if let Err(e) = self.policy_control.reload().await {
            let _ = self.policy_repo.update(&previous).await;
            return Err(e);
        }
        Ok(updated)
    }

    #[instrument(skip(self, caller))]
    pub async fn delete(&self, caller: &CallerContext, id: &CedarPolicyId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::ManagePolicies)
            .await?;

        let previous = self.policy_repo.get(id).await?;
        self.policy_repo.delete(id).await?;

        if let Err(e) = self.policy_control.reload().await {
            let _ = self.policy_repo.create(&previous).await;
            return Err(e);
        }
        Ok(())
    }
}

/// Identify the actor for the `created_by` audit field. After a passing
/// `ManagePolicies` check the caller is a user or service (never anonymous).
fn caller_id(caller: &CallerContext) -> String {
    match caller {
        CallerContext::User(id) => id.as_str().to_string(),
        CallerContext::App(id) => id.as_str().to_string(),
        CallerContext::Service(svc) => svc.as_str().to_string(),
        CallerContext::Anonymous => "anonymous".to_string(),
    }
}
