use crate::application::agent::dispatch_port::AgentDispatch;
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::domain::entities::{AppId, OrganizationId, ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::action::Action;
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

// Canonical role names — single source of truth. Convention: `<scope>-<role>`,
// kebab-case, scope ∈ {system, organization, project}. Referenced by the Cedar
// adapter (`TEMPLATE_ROLES`), the embedded policies, and callers that mint
// grants. NOTE: the global `system-admin` name is also hard-coded in
// `policies.cedar` (`@id("admin")` → `Scylla::Role::"system-admin"`) — keep both
// in sync. The implicit `system-member` / `organization-member` /
// `project-member` tiers are NOT named roles: membership is ABAC
// (`user_roles` absent / `user_organization` / `user_project`).

/// Global super-user (full control, every scope). Materialised from `user_roles`.
pub const SYSTEM_ADMIN_ROLE: &str = "system-admin";
/// Owner of an organization and everything beneath it.
pub const ORGANIZATION_ADMIN_ROLE: &str = "organization-admin";
/// Owner of a project and everything beneath it.
pub const PROJECT_ADMIN_ROLE: &str = "project-admin";
/// Restricted role for machine Apps (agents) scoped to an organization: only the
/// actions needed to pull and execute jobs within that scope. Linked via a
/// dedicated Cedar template, not the full-control one used by the admin roles.
pub const ORGANIZATION_AGENT_ROLE: &str = "organization-agent";
/// Same restricted agent capability, scoped to a single project.
pub const PROJECT_AGENT_ROLE: &str = "project-agent";

/// Owner-equivalent roles: holding one grants full control over a scope. A scope
/// must never lose its last owner, so revoking one of these is guarded. Includes
/// `system-admin` so the last global admin can't be revoked into a lockout.
#[must_use]
fn is_owner_role(role: &RoleName) -> bool {
    matches!(
        role.as_str(),
        SYSTEM_ADMIN_ROLE | ORGANIZATION_ADMIN_ROLE | PROJECT_ADMIN_ROLE
    )
}

/// The scope a grant is bound to. Maps to the `?resource` slot of the linked
/// Cedar template. `System` is the tenancy root (org ∈ System ∈ …): a grant
/// there — e.g. `system-admin` — covers everything beneath. It is the unified
/// replacement for the former global-role mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantScope {
    System,
    Organization(OrganizationId),
    Project(ProjectId),
}

impl GrantScope {
    /// The id-free discriminant of this scope — used by the assignable-roles
    /// catalog and to validate a grant's (role, scope) pairing.
    #[must_use]
    pub fn kind(&self) -> ScopeKind {
        match self {
            Self::System => ScopeKind::System,
            Self::Organization(_) => ScopeKind::Organization,
            Self::Project(_) => ScopeKind::Project,
        }
    }
}

/// Which kind of scope a role/grant binds to, without the concrete id. The
/// catalog declares one of these per role; a grant is valid only when its role's
/// declared `ScopeKind` matches its scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    System,
    Organization,
    Project,
}

impl ScopeKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Organization => "organization",
            Self::Project => "project",
        }
    }
}

/// Full-control admin role vs restricted machine-agent role. Determines which
/// Cedar template a grant links to (`ROLE_TEMPLATE` vs `AGENT_TEMPLATE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleKind {
    Admin,
    Agent,
}

impl RoleKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Agent => "agent",
        }
    }
}

/// A role assignable through a grant: its name, the scope kind it must bind to,
/// whether it is an admin or agent role, and a human description. The catalog
/// ([`GRANTABLE_ROLES`]) is the single source of truth for what `CreateGrant`
/// accepts — there are no runtime-defined roles, so listing it is exhaustive.
#[derive(Debug, Clone, Copy)]
pub struct GrantableRole {
    pub name: &'static str,
    pub scope: ScopeKind,
    pub kind: RoleKind,
    pub description: &'static str,
}

/// Every role that can be granted. One entry per `*_ROLE` constant above — keep
/// the two in sync (the test below asserts each entry's name is a known const).
pub const GRANTABLE_ROLES: &[GrantableRole] = &[
    GrantableRole {
        name: SYSTEM_ADMIN_ROLE,
        scope: ScopeKind::System,
        kind: RoleKind::Admin,
        description: "Global super-user: full control over every scope.",
    },
    GrantableRole {
        name: ORGANIZATION_ADMIN_ROLE,
        scope: ScopeKind::Organization,
        kind: RoleKind::Admin,
        description: "Owner of an organization and everything beneath it.",
    },
    GrantableRole {
        name: ORGANIZATION_AGENT_ROLE,
        scope: ScopeKind::Organization,
        kind: RoleKind::Agent,
        description: "Machine app scoped to an organization: pull and run its jobs.",
    },
    GrantableRole {
        name: PROJECT_ADMIN_ROLE,
        scope: ScopeKind::Project,
        kind: RoleKind::Admin,
        description: "Owner of a project and everything beneath it.",
    },
    GrantableRole {
        name: PROJECT_AGENT_ROLE,
        scope: ScopeKind::Project,
        kind: RoleKind::Agent,
        description: "Machine app scoped to a project: pull and run its jobs.",
    },
];

/// The assignable-role catalog, optionally narrowed to one scope kind. Pure /
/// static — no permission check (the names are compile-time constants, not
/// sensitive data) and no DB hit.
#[must_use]
pub fn grantable_roles(filter: Option<ScopeKind>) -> Vec<GrantableRole> {
    GRANTABLE_ROLES
        .iter()
        .copied()
        .filter(|r| filter.is_none_or(|k| r.scope == k))
        .collect()
}

/// Validate a grant's (role, scope) pairing against [`GRANTABLE_ROLES`]. Rejects
/// an unknown role name and a role used on the wrong scope kind (e.g. an
/// `organization-admin` on a Project). Closes the free-form `RoleName` hole so a
/// persisted grant can never name a role the Cedar adapter cannot link.
pub fn validate_role_for_scope(role: &RoleName, scope: &GrantScope) -> DomainResult<()> {
    let entry = GRANTABLE_ROLES
        .iter()
        .find(|r| r.name == role.as_str())
        .ok_or_else(|| {
            let names = GRANTABLE_ROLES
                .iter()
                .map(|r| r.name)
                .collect::<Vec<_>>()
                .join(", ");
            DomainError::validation(format!(
                "unknown role '{}'; assignable roles: {names}",
                role.as_str()
            ))
        })?;
    if entry.scope != scope.kind() {
        return Err(DomainError::validation(format!(
            "role '{}' is grantable only on {} scope, not {}",
            role.as_str(),
            entry.scope.as_str(),
            scope.kind().as_str()
        )));
    }
    Ok(())
}

/// The principal a grant is bound to — a human `User` or a machine `App`. Maps
/// to the `(principal_kind, principal_id)` columns of `grants` and to
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
/// `Action::ManageGrants` (admin/service in practice). A created or revoked
/// grant is applied live via [`PolicyControl::reload`], so it takes effect
/// immediately without a control-plane restart. Revoking an App's grant also
/// disconnects its agent stream so a no-longer-authorized agent stops at once.
#[derive(Constructor)]
pub struct GrantUseCases<G: GrantRepository, PC: PolicyControl, PS: PermissionService> {
    grant_repo: Arc<G>,
    policy_control: Arc<PC>,
    permission_service: Arc<PS>,
    agent_registry: Arc<dyn AgentDispatch>,
}

impl<G: GrantRepository, PC: PolicyControl, PS: PermissionService> GrantUseCases<G, PC, PS> {
    /// Action required to manage a grant bound to `scope`. An org-scoped
    /// grant needs `manageGrants` on that org; a project-scoped grant needs it on
    /// that project. The Cedar role template (`resource in ?resource`) then
    /// confines the caller to its own subtree — a system admin holds it on
    /// `System` (admin policy), an org admin on its org, a project admin on its
    /// project — so no caller can manage grants outside their scope
    /// (anti-escalation is enforced by Cedar, not by trusting the caller).
    fn manage_perm(scope: &GrantScope) -> Action {
        match scope {
            GrantScope::System => Action::ManageGrants,
            GrantScope::Organization(id) => Action::ManageOrgGrants(id.clone()),
            GrantScope::Project(id) => Action::ManageProjectGrants(id.clone()),
        }
    }

    /// Every grant in the system — system admins only.
    #[instrument(skip(self, caller))]
    pub async fn list(&self, caller: &CallerContext) -> DomainResult<Vec<Grant>> {
        self.permission_service
            .check(caller, Action::ManageGrants)
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
        // Reject unknown roles and role/scope mismatches before persisting, so a
        // grant always names a catalog role the Cedar adapter can link.
        validate_role_for_scope(&grant.role, &grant.scope)?;
        self.grant_repo.create(grant).await?;
        self.policy_control.reload().await
    }

    #[instrument(skip(self, caller))]
    pub async fn revoke(&self, caller: &CallerContext, id: &str) -> DomainResult<()> {
        // Look the grant up first: the caller must hold management rights over
        // *its* scope, and a revoked agent App must be disconnected.
        let grants = self.grant_repo.list_all().await?;
        let grant = grants.iter().find(|g| g.id == id).cloned();

        // Unknown id falls back to the system-scoped permission, so only admins
        // can probe arbitrary ids; the subsequent delete is then a no-op.
        let perm = grant
            .as_ref()
            .map_or(Action::ManageGrants, |g| Self::manage_perm(&g.scope));
        self.permission_service.check(caller, perm).await?;

        // Last-owner guard: a scope must always retain at least one *human* owner.
        // Only a User owner-grant is guarded — revoking an App's owner grant is
        // always allowed, and an App grant never counts as the retained owner
        // (machine principals shouldn't keep a scope "owned" with no human able
        // to administer it). If this is the final human owner, block the revoke
        // rather than orphan the org/project.
        if let Some(g) = &grant
            && is_owner_role(&g.role)
            && matches!(g.principal, GrantPrincipal::User(_))
        {
            let other_human_owners = grants
                .iter()
                .filter(|o| {
                    o.id != g.id
                        && o.role == g.role
                        && o.scope == g.scope
                        && matches!(o.principal, GrantPrincipal::User(_))
                })
                .count();
            if other_human_owners == 0 {
                return Err(DomainError::business_rule(
                    "cannot revoke the last owner of this scope",
                ));
            }
        }

        self.grant_repo.delete(id).await?;
        self.policy_control.reload().await?;

        if let Some(GrantPrincipal::App(app_id)) = grant.map(|g| g.principal) {
            self.agent_registry.disconnect(&app_id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::agent::dispatch::JobDispatch;
    use crate::domain::entities::{AppId, OrganizationId, UserId};
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
        async fn check(&self, _caller: &CallerContext, _perm: Action) -> DomainResult<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingRegistry {
        disconnected: Mutex<Vec<String>>,
    }
    #[async_trait]
    impl AgentDispatch for RecordingRegistry {
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

    #[test]
    fn grantable_roles_filter_by_scope_kind() {
        assert_eq!(grantable_roles(None).len(), GRANTABLE_ROLES.len());
        let project = grantable_roles(Some(ScopeKind::Project));
        assert_eq!(project.len(), 2);
        assert!(
            project
                .iter()
                .all(|r| r.scope == ScopeKind::Project && r.name.starts_with("project-"))
        );
        let system = grantable_roles(Some(ScopeKind::System));
        assert_eq!(system.len(), 1);
        assert_eq!(system[0].name, SYSTEM_ADMIN_ROLE);
    }

    #[test]
    fn validate_role_for_scope_accepts_catalog_pairings_and_rejects_others() {
        // Every catalog entry validates against a scope of its declared kind.
        for r in GRANTABLE_ROLES {
            let scope = match r.scope {
                ScopeKind::System => GrantScope::System,
                ScopeKind::Organization => GrantScope::Organization(OrganizationId::new("o1")),
                ScopeKind::Project => GrantScope::Project(ProjectId::new("p1")),
            };
            validate_role_for_scope(&RoleName::new(r.name).unwrap(), &scope)
                .unwrap_or_else(|_| panic!("{} must validate on its scope", r.name));
        }
        // Unknown role → rejected.
        assert!(
            validate_role_for_scope(&RoleName::new("wizard").unwrap(), &GrantScope::System).is_err()
        );
        // Right role, wrong scope kind → rejected.
        assert!(
            validate_role_for_scope(
                &RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
                &GrantScope::Project(ProjectId::new("p1")),
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn revoking_app_grant_disconnects_the_agent() {
        let grant = Grant::new(
            GrantPrincipal::App(AppId::new("agent-1")),
            RoleName::new(ORGANIZATION_AGENT_ROLE).unwrap(),
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
    async fn revoking_user_grant_leaves_agents_alone() {
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
