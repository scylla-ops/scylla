use crate::application::caller::CallerContext;
use crate::application::permission::policy::PolicyControl;
use crate::application::permission::service::PermissionService;
use crate::application::worker::dispatch_port::WorkerDispatch;
use crate::domain::entities::{AppId, OrganizationId, ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

/// Role names that map to a linkable Cedar template. Single source of truth,
/// referenced by the Cedar adapter (`TEMPLATE_ROLES`) and by callers that mint
/// grants (e.g. signup grants the org creator [`ORGANIZATION_ADMIN_ROLE`]).
pub const ORGANIZATION_ADMIN_ROLE: &str = "organization-admin";
pub const PROJECT_ADMIN_ROLE: &str = "project-admin";
/// Restricted role for machine Apps (agents): only the actions needed to pull
/// and execute jobs within a scope. Linked via a dedicated Cedar template, not
/// the full-control one used by the admin roles.
pub const WORKER_ROLE: &str = "worker";

/// Owner-equivalent roles: holding one grants full control over a scope. A scope
/// must never lose its last owner, so revoking one of these is guarded.
#[must_use]
fn is_owner_role(role: &RoleName) -> bool {
    matches!(role.as_str(), ORGANIZATION_ADMIN_ROLE | PROJECT_ADMIN_ROLE)
}

/// The scope a grant is bound to. Maps to the `?resource` slot of the linked
/// Cedar template (e.g. `Project::"X"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantScope {
    Organization(OrganizationId),
    Project(ProjectId),
}

/// The principal a grant is bound to — a human `User` or a machine `App`. Maps
/// to the `(principal_kind, principal_id)` columns of `permission_grants` and to
/// the Cedar `?principal` slot (`Scylla::User` / `Scylla::App`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantPrincipal {
    User(UserId),
    App(AppId),
}

impl GrantPrincipal {
    /// Persistence discriminant — the `principal_kind` column value.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::User(_) => "user",
            Self::App(_) => "app",
        }
    }

    /// The principal's raw id — the `principal_id` column value.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::User(id) => id.as_str(),
            Self::App(id) => id.as_str(),
        }
    }
}

/// An explicit, scoped role assignment — "principal P holds role R within scope
/// S". Each grant materialises as one linked Cedar template instance at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub id: String,
    pub principal: GrantPrincipal,
    pub role: RoleName,
    pub scope: GrantScope,
}

impl Grant {
    #[must_use]
    pub fn new(principal: GrantPrincipal, role: RoleName, scope: GrantScope) -> Self {
        Self {
            id: ulid::Ulid::new().to_string().to_lowercase(),
            principal,
            role,
            scope,
        }
    }
}

/// Persistence for explicit scoped grants. Read at `CedarPermissionService`
/// construction to link template instances; mutated by `GrantUseCases`.
#[async_trait]
pub trait GrantRepository: Send + Sync {
    async fn list_all(&self) -> DomainResult<Vec<Grant>>;
    async fn create(&self, grant: &Grant) -> DomainResult<()>;
    async fn delete(&self, id: &str) -> DomainResult<()>;
}

/// Admin-only management of scoped grants. Every method is gated by
/// `Permission::ManageGrants` (admin/service in practice). A created or revoked
/// grant is applied live via [`PolicyControl::reload`], so it takes effect
/// immediately without a control-plane restart. Revoking an App's grant also
/// disconnects its worker stream so a no-longer-authorized agent stops at once.
#[derive(Constructor)]
pub struct GrantUseCases<G: GrantRepository, PC: PolicyControl, PS: PermissionService> {
    grant_repo: Arc<G>,
    policy_control: Arc<PC>,
    permission_service: Arc<PS>,
    worker_registry: Arc<dyn WorkerDispatch>,
}

impl<G: GrantRepository, PC: PolicyControl, PS: PermissionService> GrantUseCases<G, PC, PS> {
    /// Permission required to manage a grant bound to `scope`. An org-scoped
    /// grant needs `manageGrants` on that org; a project-scoped grant needs it on
    /// that project. The Cedar role template (`resource in ?resource`) then
    /// confines the caller to its own subtree — a system admin holds it on
    /// `System` (admin policy), an org admin on its org, a project admin on its
    /// project — so no caller can manage grants outside their scope
    /// (anti-escalation is enforced by Cedar, not by trusting the caller).
    fn manage_perm(scope: &GrantScope) -> Permission {
        match scope {
            GrantScope::Organization(id) => Permission::ManageOrgGrants(id.clone()),
            GrantScope::Project(id) => Permission::ManageProjectGrants(id.clone()),
        }
    }

    /// Every grant in the system — system admins only.
    #[instrument(skip(self, caller))]
    pub async fn list(&self, caller: &CallerContext) -> DomainResult<Vec<Grant>> {
        self.permission_service
            .check(caller, Permission::ManageGrants)
            .await?;
        self.grant_repo.list_all().await
    }

    /// Grants bound to a specific scope — manageable by an admin of that scope
    /// (or a system admin). Backs per-org / per-project permission views.
    #[instrument(skip(self, caller))]
    pub async fn list_by_scope(
        &self,
        caller: &CallerContext,
        scope: &GrantScope,
    ) -> DomainResult<Vec<Grant>> {
        self.permission_service
            .check(caller, Self::manage_perm(scope))
            .await?;
        let grants = self.grant_repo.list_all().await?;
        Ok(grants.into_iter().filter(|g| &g.scope == scope).collect())
    }

    #[instrument(skip(self, caller))]
    pub async fn grant(&self, caller: &CallerContext, grant: &Grant) -> DomainResult<()> {
        self.permission_service
            .check(caller, Self::manage_perm(&grant.scope))
            .await?;
        self.grant_repo.create(grant).await?;
        self.policy_control.reload().await
    }

    #[instrument(skip(self, caller))]
    pub async fn revoke(&self, caller: &CallerContext, id: &str) -> DomainResult<()> {
        // Look the grant up first: the caller must hold management rights over
        // *its* scope, and a revoked worker App must be disconnected.
        let grants = self.grant_repo.list_all().await?;
        let grant = grants.iter().find(|g| g.id == id).cloned();

        // Unknown id falls back to the system-scoped permission, so only admins
        // can probe arbitrary ids; the subsequent delete is then a no-op.
        let perm = grant
            .as_ref()
            .map_or(Permission::ManageGrants, |g| Self::manage_perm(&g.scope));
        self.permission_service.check(caller, perm).await?;

        // Last-owner guard: a scope must always retain at least one owner. If
        // this grant is the final owner-role grant for its scope, block the
        // revoke rather than orphan the org/project.
        if let Some(g) = &grant
            && is_owner_role(&g.role)
        {
            let other_owners = grants
                .iter()
                .filter(|o| o.id != g.id && o.role == g.role && o.scope == g.scope)
                .count();
            if other_owners == 0 {
                return Err(DomainError::business_rule(
                    "cannot revoke the last owner of this scope",
                ));
            }
        }

        self.grant_repo.delete(id).await?;
        self.policy_control.reload().await?;

        if let Some(GrantPrincipal::App(app_id)) = grant.map(|g| g.principal) {
            self.worker_registry.disconnect(&app_id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{AppId, OrganizationId, UserId};
    use crate::domain::value_objects::pipeline::JobDispatch;
    use std::sync::Mutex;

    struct StubGrants(Vec<Grant>);
    #[async_trait]
    impl GrantRepository for StubGrants {
        async fn list_all(&self) -> DomainResult<Vec<Grant>> {
            Ok(self.0.clone())
        }
        async fn create(&self, _g: &Grant) -> DomainResult<()> {
            Ok(())
        }
        async fn delete(&self, _id: &str) -> DomainResult<()> {
            Ok(())
        }
    }

    struct StubPolicy;
    #[async_trait]
    impl PolicyControl for StubPolicy {
        async fn validate_policy(&self, _text: &str) -> DomainResult<()> {
            Ok(())
        }
        async fn reload(&self) -> DomainResult<()> {
            Ok(())
        }
    }

    struct StubPerms;
    #[async_trait]
    impl PermissionService for StubPerms {
        async fn check(&self, _caller: &CallerContext, _perm: Permission) -> DomainResult<bool> {
            Ok(true)
        }
    }

    #[derive(Default)]
    struct RecordingRegistry {
        disconnected: Mutex<Vec<String>>,
    }
    #[async_trait]
    impl WorkerDispatch for RecordingRegistry {
        fn connected(&self) -> Vec<AppId> {
            vec![]
        }
        async fn dispatch(&self, _app_id: &AppId, _d: &JobDispatch) -> DomainResult<()> {
            Ok(())
        }
        fn disconnect(&self, app_id: &AppId) {
            self.disconnected
                .lock()
                .unwrap()
                .push(app_id.as_str().to_string());
        }
    }

    fn use_cases(
        grants: Vec<Grant>,
        reg: Arc<RecordingRegistry>,
    ) -> GrantUseCases<StubGrants, StubPolicy, StubPerms> {
        GrantUseCases::new(
            Arc::new(StubGrants(grants)),
            Arc::new(StubPolicy),
            Arc::new(StubPerms),
            reg,
        )
    }

    #[tokio::test]
    async fn revoking_app_grant_disconnects_the_worker() {
        let grant = Grant::new(
            GrantPrincipal::App(AppId::new("agent-1")),
            RoleName::new(WORKER_ROLE).unwrap(),
            GrantScope::Organization(OrganizationId::new("o1")),
        );
        let reg = Arc::new(RecordingRegistry::default());
        let uc = use_cases(vec![grant.clone()], reg.clone());

        uc.revoke(&CallerContext::User(UserId::new("admin")), &grant.id)
            .await
            .unwrap();

        assert_eq!(reg.disconnected.lock().unwrap().as_slice(), ["agent-1"]);
    }

    #[tokio::test]
    async fn cannot_revoke_last_owner_of_scope() {
        // The sole org-admin of an org may not be revoked — it would orphan it.
        let grant = Grant::new(
            GrantPrincipal::User(UserId::new("u1")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            GrantScope::Organization(OrganizationId::new("o1")),
        );
        let reg = Arc::new(RecordingRegistry::default());
        let uc = use_cases(vec![grant.clone()], reg);

        assert!(
            uc.revoke(&CallerContext::User(UserId::new("admin")), &grant.id)
                .await
                .is_err(),
            "revoking the last owner must be blocked"
        );
    }

    #[tokio::test]
    async fn can_revoke_owner_when_another_exists() {
        let scope = GrantScope::Organization(OrganizationId::new("o1"));
        let g1 = Grant::new(
            GrantPrincipal::User(UserId::new("u1")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            scope.clone(),
        );
        let g2 = Grant::new(
            GrantPrincipal::User(UserId::new("u2")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            scope,
        );
        let reg = Arc::new(RecordingRegistry::default());
        let uc = use_cases(vec![g1.clone(), g2], reg);

        assert!(
            uc.revoke(&CallerContext::User(UserId::new("admin")), &g1.id)
                .await
                .is_ok(),
            "revoking one of two owners is allowed"
        );
    }

    #[tokio::test]
    async fn revoking_user_grant_leaves_workers_alone() {
        let grant = Grant::new(
            GrantPrincipal::User(UserId::new("u1")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            GrantScope::Organization(OrganizationId::new("o1")),
        );
        // A co-owner so the last-owner guard permits the revoke.
        let co_owner = Grant::new(
            GrantPrincipal::User(UserId::new("u2")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            GrantScope::Organization(OrganizationId::new("o1")),
        );
        let reg = Arc::new(RecordingRegistry::default());
        let uc = use_cases(vec![grant.clone(), co_owner], reg.clone());

        uc.revoke(&CallerContext::User(UserId::new("admin")), &grant.id)
            .await
            .unwrap();

        assert!(reg.disconnected.lock().unwrap().is_empty());
    }
}
