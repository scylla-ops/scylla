use crate::application::agent::dispatch_port::AgentDispatch;
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::role::{FULL_CONTROL, RoleRepository};
use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::domain::entities::{AppId, OrganizationId, ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::{Permission, is_known_permission};
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;
use derive_more::Constructor;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use tracing::instrument;

// Canonical builtin role keys — the stable ids grants reference and the default
// seed inserts. Convention: `<scope>-<role>`, kebab-case, scope ∈ {system,
// organization, project}. The live Cedar policy bodies are generated per role
// from the `roles` table (see `cedar_permission_service`), not hard-coded here.
// The implicit `system-member` / `organization-member` / `project-member` tiers
// are NOT named roles: membership is ABAC (`user_organization` / `user_project`).

/// Global super-user (full control, every scope), via a grant on the System scope.
pub const SYSTEM_ADMIN_ROLE: &str = "system-admin";
/// Owner of an organization and everything beneath it.
pub const ORGANIZATION_ADMIN_ROLE: &str = "organization-admin";
/// Owner of a project and everything beneath it.
pub const PROJECT_ADMIN_ROLE: &str = "project-admin";
/// Restricted role for machine Apps (agents) scoped to an organization: only the
/// permissions needed to pull and execute jobs within that scope (read pipeline,
/// execute job, write job status/log), held in the role's `role_permissions`.
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
pub enum Scope {
    System,
    Organization(OrganizationId),
    Project(ProjectId),
}

impl Scope {
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

    /// Containment depth in the scope hierarchy (System is broadest = 0).
    fn depth(self) -> u8 {
        match self {
            Self::System => 0,
            Self::Organization => 1,
            Self::Project => 2,
        }
    }

    /// Whether a grant at this scope covers resources whose home scope is
    /// `inner` — true when this scope is `inner` or one of its ancestors
    /// (System ⊃ Organization ⊃ Project). A grant authorises within its scope's
    /// subtree, so a permission is usable in a role iff the role's scope covers
    /// the permission's home scope.
    #[must_use]
    pub fn covers(self, inner: ScopeKind) -> bool {
        self.depth() <= inner.depth()
    }
}

/// Full-control admin role vs restricted machine-agent role. Used by the
/// compile-time grantable-role catalog ([`GRANTABLE_ROLES`]) to describe a
/// builtin; the live Cedar policy bodies are now generated per role from the
/// `roles` table (full control → unconstrained action; otherwise the role's
/// explicit permission keys), not from this kind.
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
pub fn validate_role_for_scope(role: &RoleName, scope: &Scope) -> DomainResult<()> {
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

/// Validate a direct permission grant's key against the permission catalog, so a
/// persisted grant can never name a permission the Cedar adapter cannot emit.
pub fn validate_permission_key(key: &str) -> DomainResult<()> {
    if is_known_permission(key) {
        Ok(())
    } else {
        Err(DomainError::validation(format!(
            "unknown permission '{key}'"
        )))
    }
}

/// The principal a grant is bound to — a human `User` or a machine `App`. Maps
/// to the `(principal_kind, principal_id)` columns of `grants` and to
/// the Cedar `?principal` slot (`Scylla::User` / `Scylla::App`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Principal {
    User(UserId),
    App(AppId),
}

impl Principal {
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

/// What a grant confers: a whole role, or a single permission. A direct
/// permission grant is additive to the principal's role-derived permissions
/// (e.g. "Alice may runPipeline within Org A" on top of whatever roles she has).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantTarget {
    /// A role, referenced by its id (== key for builtins).
    Role(RoleName),
    /// A single permission, by its [`crate::domain::value_objects::permission::Permission::key`].
    Permission(String),
}

impl GrantTarget {
    /// Persistence discriminant — the `target_kind` column value.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Role(_) => "role",
            Self::Permission(_) => "permission",
        }
    }

    /// The stored `target` column value — a role id or a permission key.
    #[must_use]
    pub fn value(&self) -> &str {
        match self {
            Self::Role(role) => role.as_str(),
            Self::Permission(key) => key.as_str(),
        }
    }
}

/// An explicit, scoped grant — "principal P holds TARGET within scope S", where
/// the target is a role or a single permission. A role grant materialises as a
/// linked Cedar template instance; a permission grant as a direct permit policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub id: String,
    pub principal: Principal,
    pub target: GrantTarget,
    pub scope: Scope,
}

impl Grant {
    /// A grant of a whole role.
    #[must_use]
    pub fn new(principal: Principal, role: RoleName, scope: Scope) -> Self {
        Self::with_target(principal, GrantTarget::Role(role), scope)
    }

    /// A grant of a single permission (by its `Permission::key()`), additive to
    /// the principal's role-derived permissions.
    #[must_use]
    pub fn with_permission(
        principal: Principal,
        permission: impl Into<String>,
        scope: Scope,
    ) -> Self {
        Self::with_target(principal, GrantTarget::Permission(permission.into()), scope)
    }

    #[must_use]
    fn with_target(principal: Principal, target: GrantTarget, scope: Scope) -> Self {
        Self {
            id: ulid::Ulid::new().to_string().to_lowercase(),
            principal,
            target,
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
/// disconnects its agent stream so a no-longer-authorized agent stops at once.
#[derive(Constructor)]
pub struct GrantUseCases<G: GrantRepository, PC: PolicyControl, PS: PermissionService> {
    grant_repo: Arc<G>,
    role_repo: Arc<dyn RoleRepository>,
    policy_control: Arc<PC>,
    permission_service: Arc<PS>,
    agent_registry: Arc<dyn AgentDispatch>,
}

/// What a principal holds at a scope, for the anti-escalation check.
enum Holding {
    /// Confers every permission (a `*` role/grant).
    Full,
    /// An explicit set of permission keys.
    Keys(BTreeSet<String>),
}

impl<G: GrantRepository, PC: PolicyControl, PS: PermissionService> GrantUseCases<G, PC, PS> {
    /// Permission required to manage a grant bound to `scope`. An org-scoped
    /// grant needs `manageGrants` on that org; a project-scoped grant needs it on
    /// that project. The Cedar role template (`resource in ?resource`) then
    /// confines the caller to its own subtree — a system admin holds it on
    /// `System` (admin policy), an org admin on its org, a project admin on its
    /// project — so no caller can manage grants outside their scope
    /// (anti-escalation is enforced by Cedar, not by trusting the caller).
    fn manage_perm(scope: &Scope) -> Permission {
        match scope {
            Scope::System => Permission::ManageGrants,
            Scope::Organization(id) => Permission::ManageOrgGrants(id.clone()),
            Scope::Project(id) => Permission::ManageProjectGrants(id.clone()),
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
        scope: &Scope,
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
        // Validate the target before persisting so a stored grant is always
        // emittable into Cedar: a role grant names an existing role valid at its
        // scope; a permission grant names a known permission key.
        match &grant.target {
            GrantTarget::Role(role) => self.validate_role_target(role, &grant.scope).await?,
            GrantTarget::Permission(key) => validate_permission_key(key)?,
        }
        self.check_no_escalation(caller, grant).await?;
        self.grant_repo.create(grant).await?;
        self.policy_control.reload().await
    }

    /// Validate a role grant against the DB role catalog: the role must exist
    /// (builtin *or* custom) and declare the scope kind the grant binds to.
    /// Supersedes the static, builtin-only [`validate_role_for_scope`] so a role
    /// created through `RoleService` becomes grantable — the rest of the pipeline
    /// (anti-escalation expansion via [`Self::target_keys`] and Cedar emission)
    /// already resolves any role by id. Builtins live in the same table (id ==
    /// key), so they validate through this path unchanged.
    async fn validate_role_target(&self, role: &RoleName, scope: &Scope) -> DomainResult<()> {
        let found = self.role_repo.get(role.as_str()).await?.ok_or_else(|| {
            DomainError::validation(format!("unknown role '{}'", role.as_str()))
        })?;
        if found.scope != scope.kind() {
            return Err(DomainError::validation(format!(
                "role '{}' is grantable only on {} scope, not {}",
                role.as_str(),
                found.scope.as_str(),
                scope.kind().as_str()
            )));
        }
        Ok(())
    }

    /// Anti-escalation: a delegator may only confer permissions it already holds
    /// at the grant's scope. Without this, a principal granted only
    /// `manageOrgGrants` (a narrow custom role) could grant itself
    /// `organization-admin` (full control) — lateral movement. Internal services
    /// bypass (they act as the system). Enforced for System and Organization
    /// scopes, where a scope's ancestors are statically known (System covers
    /// everything; an org's only ancestor is System). Project-scope grants are
    /// not subset-checked yet (smallest blast radius) — they stay gated by
    /// `manageProjectGrants`.
    async fn check_no_escalation(&self, caller: &CallerContext, grant: &Grant) -> DomainResult<()> {
        let principal = match caller {
            CallerContext::User(id) => Principal::User(id.clone()),
            CallerContext::App(id) => Principal::App(id.clone()),
            // Services act as the system; Anonymous is already denied upstream.
            CallerContext::Service(_) | CallerContext::Anonymous => return Ok(()),
        };
        if matches!(grant.scope, Scope::Project(_)) {
            return Ok(());
        }

        let allowed = match (
            self.holding_at(&principal, &grant.scope).await?,
            self.target_keys(&grant.target).await?,
        ) {
            (Holding::Full, _) => true,
            // Can't confer full control without holding it.
            (Holding::Keys(_), None) => false,
            (Holding::Keys(have), Some(want)) => want.iter().all(|k| have.contains(k)),
        };
        if allowed {
            Ok(())
        } else {
            Err(DomainError::business_rule(
                "cannot grant permissions you do not hold at this scope (no privilege escalation)",
            ))
        }
    }

    /// The permission keys a grant target confers, or `None` for full control.
    async fn target_keys(&self, target: &GrantTarget) -> DomainResult<Option<BTreeSet<String>>> {
        match target {
            GrantTarget::Permission(key) => Ok(Some(BTreeSet::from([key.clone()]))),
            GrantTarget::Role(role) => match self.role_repo.get(role.as_str()).await? {
                Some(r) if r.is_full_control() => Ok(None),
                Some(r) => Ok(Some(r.permissions.into_iter().collect())),
                // Unknown role confers nothing; an empty set is trivially a subset.
                None => Ok(Some(BTreeSet::new())),
            },
        }
    }

    /// What `principal` holds applicable to `scope` — its grants at that scope
    /// plus System (which covers everything). `Full` if any confers full control.
    async fn holding_at(&self, principal: &Principal, scope: &Scope) -> DomainResult<Holding> {
        let role_perms: HashMap<String, Vec<String>> = self
            .role_repo
            .list_all()
            .await?
            .into_iter()
            .map(|r| (r.id, r.permissions))
            .collect();
        let grants = self.grant_repo.list_all().await?;

        let mut keys = BTreeSet::new();
        for g in grants.iter().filter(|g| g.principal == *principal) {
            if !(matches!(g.scope, Scope::System) || &g.scope == scope) {
                continue;
            }
            let perms: Vec<String> = match &g.target {
                GrantTarget::Role(role) => {
                    role_perms.get(role.as_str()).cloned().unwrap_or_default()
                }
                GrantTarget::Permission(key) => vec![key.clone()],
            };
            if perms.iter().any(|p| p == FULL_CONTROL) {
                return Ok(Holding::Full);
            }
            keys.extend(perms);
        }
        Ok(Holding::Keys(keys))
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
            .map_or(Permission::ManageGrants, |g| Self::manage_perm(&g.scope));
        self.permission_service.check(caller, perm).await?;

        // Last-owner guard: a scope must always retain at least one *human* owner.
        // Only a User owner-grant is guarded — revoking an App's owner grant is
        // always allowed, and an App grant never counts as the retained owner
        // (machine principals shouldn't keep a scope "owned" with no human able
        // to administer it). If this is the final human owner, block the revoke
        // rather than orphan the org/project.
        if let Some(g) = &grant
            && let GrantTarget::Role(role) = &g.target
            && is_owner_role(role)
            && matches!(g.principal, Principal::User(_))
        {
            let other_human_owners = grants
                .iter()
                .filter(|o| {
                    o.id != g.id
                        && o.target == g.target
                        && o.scope == g.scope
                        && matches!(o.principal, Principal::User(_))
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

        if let Some(Principal::App(app_id)) = grant.map(|g| g.principal) {
            self.agent_registry.disconnect(&app_id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::agent::dispatch::JobDispatch;
    use crate::application::authz::role::Role;
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
        async fn check(&self, _caller: &CallerContext, _perm: Permission) -> DomainResult<()> {
            Ok(())
        }
    }

    struct StubRoles(Vec<Role>);
    #[async_trait]
    impl RoleRepository for StubRoles {
        async fn list_all(&self) -> DomainResult<Vec<Role>> {
            Ok(self.0.clone())
        }
        async fn get(&self, id: &str) -> DomainResult<Option<Role>> {
            Ok(self.0.iter().find(|r| r.id == id).cloned())
        }
        async fn create(&self, _r: &Role) -> DomainResult<()> {
            Ok(())
        }
        async fn update(&self, _r: &Role) -> DomainResult<()> {
            Ok(())
        }
        async fn delete(&self, _id: &str) -> DomainResult<()> {
            Ok(())
        }
    }

    /// Build a role used as escalation-test fixture data.
    fn test_role(id: &str, scope: ScopeKind, permissions: &[&str]) -> Role {
        Role {
            id: id.to_string(),
            key: Some(id.to_string()),
            name: id.to_string(),
            description: String::new(),
            scope,
            owner_org: None,
            builtin: true,
            permissions: permissions.iter().map(ToString::to_string).collect(),
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
        use_cases_with(grants, vec![], reg)
    }

    fn use_cases_with(
        grants: Vec<Grant>,
        roles: Vec<Role>,
        reg: Arc<RecordingRegistry>,
    ) -> GrantUseCases<StubGrants, StubPolicy, StubPerms> {
        GrantUseCases::new(
            Arc::new(StubGrants(grants)),
            Arc::new(StubRoles(roles)),
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
                ScopeKind::System => Scope::System,
                ScopeKind::Organization => Scope::Organization(OrganizationId::new("o1")),
                ScopeKind::Project => Scope::Project(ProjectId::new("p1")),
            };
            validate_role_for_scope(&RoleName::new(r.name).unwrap(), &scope)
                .unwrap_or_else(|_| panic!("{} must validate on its scope", r.name));
        }
        // Unknown role → rejected.
        assert!(
            validate_role_for_scope(&RoleName::new("wizard").unwrap(), &Scope::System).is_err()
        );
        // Right role, wrong scope kind → rejected.
        assert!(
            validate_role_for_scope(
                &RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
                &Scope::Project(ProjectId::new("p1")),
            )
            .is_err()
        );
    }

    #[test]
    fn validate_permission_key_accepts_catalog_rejects_unknown() {
        // A real permission key validates; an invented one is rejected, so a
        // direct permission grant can never name something Cedar can't emit.
        assert!(validate_permission_key("runPipeline").is_ok());
        assert!(validate_permission_key("readJob").is_ok());
        assert!(validate_permission_key("flyToTheMoon").is_err());
    }

    #[test]
    fn grant_target_constructors_set_kind_and_value() {
        let role = Grant::new(
            Principal::User(UserId::new("u1")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            Scope::Organization(OrganizationId::new("o1")),
        );
        assert_eq!(role.target.kind(), "role");
        assert_eq!(role.target.value(), ORGANIZATION_ADMIN_ROLE);

        let perm = Grant::with_permission(
            Principal::User(UserId::new("u1")),
            "runPipeline",
            Scope::Organization(OrganizationId::new("o1")),
        );
        assert_eq!(perm.target.kind(), "permission");
        assert_eq!(perm.target.value(), "runPipeline");
    }

    #[tokio::test]
    async fn anti_escalation_blocks_granting_more_than_you_hold() {
        let org = Scope::Organization(OrganizationId::new("o1"));
        // Bob holds only `manageOrgGrants` on the org; Carol holds the
        // full-control organization-admin role there.
        let roles = vec![test_role(
            ORGANIZATION_ADMIN_ROLE,
            ScopeKind::Organization,
            &[FULL_CONTROL],
        )];
        let uc = use_cases_with(
            vec![
                Grant::with_permission(
                    Principal::User(UserId::new("bob")),
                    "manageOrgGrants",
                    org.clone(),
                ),
                Grant::new(
                    Principal::User(UserId::new("carol")),
                    RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
                    org.clone(),
                ),
            ],
            roles,
            Arc::new(RecordingRegistry::default()),
        );
        let bob = CallerContext::User(UserId::new("bob"));
        let carol = CallerContext::User(UserId::new("carol"));

        // Bob (only manageOrgGrants) cannot grant the full-control org-admin role.
        assert!(
            uc.grant(
                &bob,
                &Grant::new(
                    Principal::User(UserId::new("alice")),
                    RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
                    org.clone(),
                ),
            )
            .await
            .is_err(),
            "escalation to full control must be blocked",
        );

        // Bob CAN delegate a permission he himself holds.
        assert!(
            uc.grant(
                &bob,
                &Grant::with_permission(
                    Principal::User(UserId::new("alice")),
                    "manageOrgGrants",
                    org.clone(),
                ),
            )
            .await
            .is_ok(),
            "delegating a permission you hold is allowed",
        );

        // Carol (full control) may grant the org-admin role.
        assert!(
            uc.grant(
                &carol,
                &Grant::new(
                    Principal::User(UserId::new("dave")),
                    RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
                    org,
                ),
            )
            .await
            .is_ok(),
            "a full-control admin may grant",
        );
    }

    #[tokio::test]
    async fn custom_role_grantable_via_db_with_scope_check() {
        let org = Scope::Organization(OrganizationId::new("o1"));
        // A custom (non-builtin) org-scoped role, resolved from the DB by id.
        let mut custom = test_role("01customrole", ScopeKind::Organization, &["readOrganization"]);
        custom.builtin = false;
        custom.key = None;
        let admin = test_role(ORGANIZATION_ADMIN_ROLE, ScopeKind::Organization, &[FULL_CONTROL]);
        let uc = use_cases_with(
            vec![Grant::new(
                Principal::User(UserId::new("owner")),
                RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
                org.clone(),
            )],
            vec![custom, admin],
            Arc::new(RecordingRegistry::default()),
        );
        let owner = CallerContext::User(UserId::new("owner"));

        // A custom role valid at its scope is grantable (owner holds full control).
        assert!(
            uc.grant(
                &owner,
                &Grant::new(
                    Principal::User(UserId::new("alice")),
                    RoleName::new("01customrole").unwrap(),
                    org.clone(),
                ),
            )
            .await
            .is_ok(),
            "a custom role valid at its scope must be grantable",
        );

        // An unknown role id is rejected (closes the free-form RoleName hole).
        assert!(
            uc.grant(
                &owner,
                &Grant::new(
                    Principal::User(UserId::new("alice")),
                    RoleName::new("ghost").unwrap(),
                    org,
                ),
            )
            .await
            .is_err(),
            "unknown role must be rejected",
        );

        // The custom role on the wrong scope kind is rejected.
        assert!(
            uc.grant(
                &owner,
                &Grant::new(
                    Principal::User(UserId::new("alice")),
                    RoleName::new("01customrole").unwrap(),
                    Scope::Project(ProjectId::new("p1")),
                ),
            )
            .await
            .is_err(),
            "a custom role on the wrong scope kind must be rejected",
        );
    }

    #[tokio::test]
    async fn revoking_app_grant_disconnects_the_agent() {
        let grant = Grant::new(
            Principal::App(AppId::new("agent-1")),
            RoleName::new(ORGANIZATION_AGENT_ROLE).unwrap(),
            Scope::Organization(OrganizationId::new("o1")),
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
            Principal::User(UserId::new("u1")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            Scope::Organization(OrganizationId::new("o1")),
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
        let scope = Scope::Organization(OrganizationId::new("o1"));
        let g1 = Grant::new(
            Principal::User(UserId::new("u1")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            scope.clone(),
        );
        let g2 = Grant::new(
            Principal::User(UserId::new("u2")),
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
            Principal::User(UserId::new("u1")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            Scope::Organization(OrganizationId::new("o1")),
        );
        // A co-owner so the last-owner guard permits the revoke.
        let co_owner = Grant::new(
            Principal::User(UserId::new("u2")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            Scope::Organization(OrganizationId::new("o1")),
        );
        let reg = Arc::new(RecordingRegistry::default());
        let uc = use_cases(vec![grant.clone(), co_owner], reg.clone());

        uc.revoke(&CallerContext::User(UserId::new("admin")), &grant.id)
            .await
            .unwrap();

        assert!(reg.disconnected.lock().unwrap().is_empty());
    }
}
