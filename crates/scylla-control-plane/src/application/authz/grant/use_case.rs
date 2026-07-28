use super::{
    Grant, GrantRepository, Principal, Scope, is_owner_role, removal_orphans_scope,
    validate_role_in_db,
};
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::application::authz::entity_provider::AuthzEntityProvider;
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::role::{FULL_CONTROL, RoleRepository};
use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::{Permission, ResourceRef};
use crate::domain::value_objects::role::RoleName;
use derive_more::Constructor;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use tracing::instrument;

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
    /// Membership lookup backing the grantee-membership rule in [`Self::grant`].
    entity_provider: Arc<dyn AuthzEntityProvider>,
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
            Scope::System => Permission::ManageSystemGrants,
            Scope::Organization(id) => Permission::ManageOrgGrants(id.clone()),
            Scope::Project(id) => Permission::ManageProjectGrants(id.clone()),
        }
    }

    /// Every grant in the system — system admins only.
    #[instrument(skip(self, caller))]
    pub async fn list(&self, caller: &CallerContext) -> DomainResult<Vec<Grant>> {
        self.permission_service
            .check(caller, Permission::ManageSystemGrants)
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
        // Validate the role before persisting so a stored grant is always
        // emittable into Cedar: it must name an existing role valid at the
        // grant's scope.
        validate_role_in_db(&*self.role_repo, &grant.role, &grant.scope).await?;
        self.require_grantee_in_organization(grant).await?;
        self.check_no_escalation(caller, grant).await?;
        self.grant_repo.create(grant).await?;
        self.policy_control.reload().await
    }

    /// A project-scoped grant may only go to someone the organization has already
    /// accepted, meaning someone already holding a grant somewhere under it.
    ///
    /// "Already accepted" means holding a grant at this organization's own
    /// scope, which is the two-step flow the product documents: admit someone to
    /// the organization (`organization-member` is enough), then assign them to
    /// projects.
    ///
    /// This is the tenant boundary. Admitting a person requires
    /// `manageOrgGrants`, which only organization and system administrators
    /// hold; a project administrator then distributes access among those people.
    /// Without this check a project administrator could attach any account in
    /// the installation, including one belonging to another customer, and
    /// `check_no_escalation` would not catch it because it returns early for
    /// project scope.
    ///
    /// Organization- and System-scoped grants are exempt: they are how someone
    /// enters in the first place, so requiring prior access would be circular.
    /// Machine Apps are exempt too — they are owned by an organization by
    /// construction, not admitted to it.
    async fn require_grantee_in_organization(&self, grant: &Grant) -> DomainResult<()> {
        let Principal::User(_) = &grant.principal else {
            return Ok(());
        };
        let Scope::Project(project_id) = &grant.scope else {
            return Ok(());
        };
        let org_id = self
            .entity_provider
            .resource_ancestors(&ResourceRef::Project(project_id.clone()))
            .await?
            .organization
            .ok_or_else(|| DomainError::not_found("Project", project_id.to_string()))?;

        let admitted = self.grant_repo.list_all().await?.into_iter().any(|g| {
            g.principal == grant.principal
                && matches!(&g.scope, Scope::Organization(o) if o == &org_id)
        });

        if admitted {
            Ok(())
        } else {
            Err(DomainError::business_rule(
                "the user must already have access to this organization before \
                 receiving a grant on one of its projects",
            ))
        }
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
        // Services act as the system; Anonymous is already denied upstream.
        let Some(principal) = Principal::from_caller(caller) else {
            return Ok(());
        };
        if matches!(grant.scope, Scope::Project(_)) {
            return Ok(());
        }

        let allowed = match (
            self.holding_at(&principal, &grant.scope).await?,
            self.role_keys(&grant.role).await?,
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

    /// The permission keys a role confers, or `None` for full control.
    async fn role_keys(&self, role: &RoleName) -> DomainResult<Option<BTreeSet<String>>> {
        match self.role_repo.get(role.as_str()).await? {
            Some(r) if r.is_full_control() => Ok(None),
            Some(r) => Ok(Some(r.permissions.into_iter().collect())),
            // Unknown role confers nothing; an empty set is trivially a subset.
            None => Ok(Some(BTreeSet::new())),
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
            let perms: Vec<String> = role_perms.get(g.role.as_str()).cloned().unwrap_or_default();
            if perms.iter().any(|p| p == FULL_CONTROL) {
                return Ok(Holding::Full);
            }
            keys.extend(perms);
        }
        Ok(Holding::Keys(keys))
    }

    /// Strip every access a principal holds at `scope` and below it, in one
    /// operation. This is how someone leaves an organization or a project: it
    /// replaces the member-removal calls that existed when membership was a
    /// table of its own.
    ///
    /// Returns how many grants were removed, so a caller can tell "removed
    /// three" from "there was nothing to remove".
    #[instrument(skip(self, caller))]
    pub async fn revoke_all_access(
        &self,
        caller: &CallerContext,
        principal: &Principal,
        scope: &Scope,
    ) -> DomainResult<u64> {
        self.permission_service
            .check(caller, Self::manage_perm(scope))
            .await?;

        // Same rule as the per-grant revoke: a scope must always keep at least
        // one human owner, so stripping the last one is refused rather than
        // leaving the organization or project unadministered.
        let grants = self.grant_repo.list_all().await?;
        if removal_orphans_scope(&grants, scope, principal) {
            return Err(DomainError::business_rule(
                "cannot remove the last owner of this scope",
            ));
        }

        let removed = self.grant_repo.revoke_all(principal, scope).await?;
        self.policy_control.reload().await?;

        // A machine principal that just lost its access must not keep running on
        // an already-open stream.
        if let Principal::App(app_id) = principal {
            self.agent_registry.disconnect(app_id);
        }
        Ok(removed)
    }

    #[instrument(skip(self, caller))]
    pub async fn revoke(&self, caller: &CallerContext, id: &str) -> DomainResult<()> {
        // Look the grant up first: the caller must hold management rights over
        // *its* scope, and a revoked agent App must be disconnected.
        let grants = self.grant_repo.list_all().await?;
        let grant = grants.iter().find(|g| g.id == id).cloned();

        // Unknown id falls back to the system-scoped permission, so only admins
        // can probe arbitrary ids; the subsequent delete is then a no-op.
        let perm = grant.as_ref().map_or(Permission::ManageSystemGrants, |g| {
            Self::manage_perm(&g.scope)
        });
        self.permission_service.check(caller, perm).await?;

        // Last-owner guard: a scope must always retain at least one *human* owner.
        // Only a User owner-grant is guarded — revoking an App's owner grant is
        // always allowed, and an App grant never counts as the retained owner
        // (machine principals shouldn't keep a scope "owned" with no human able
        // to administer it). If this is the final human owner, block the revoke
        // rather than orphan the org/project.
        if let Some(g) = &grant
            && is_owner_role(&g.role)
            && matches!(g.principal, Principal::User(_))
        {
            let other_human_owners = grants
                .iter()
                .filter(|o| {
                    o.id != g.id
                        && o.role == g.role
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
    use super::super::*;
    use super::*;
    use crate::application::agent::dispatch::JobDispatch;
    use crate::application::authz::entity_provider::ResourceAncestors;
    use crate::application::authz::role::Role;
    use crate::application::caller::ServiceIdentity;
    use crate::domain::entities::{AppId, OrganizationId, UserId};
    use async_trait::async_trait;
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
        async fn revoke_all(&self, _p: &Principal, _s: &Scope) -> DomainResult<u64> {
            Ok(0)
        }
    }

    struct StubPolicy;
    #[async_trait]
    impl PolicyControl for StubPolicy {
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

    /// Every project in these tests resolves under the single org `o1`.
    struct StubAncestry;
    #[async_trait]
    impl AuthzEntityProvider for StubAncestry {
        async fn resource_ancestors(
            &self,
            _resource: &ResourceRef,
        ) -> DomainResult<ResourceAncestors> {
            Ok(ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                ..Default::default()
            })
        }
        async fn app_is_active(&self, _app: &AppId) -> DomainResult<bool> {
            Ok(true)
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

    impl RecordingRegistry {
        fn disconnected(&self) -> Vec<String> {
            self.disconnected.lock().unwrap().clone()
        }
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
        fn in_flight(&self, _app_id: &AppId) -> usize {
            0
        }
        fn release(&self, _app_id: &AppId) {}
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
            Arc::new(StubAncestry),
        )
    }

    #[test]
    fn grantable_roles_filter_by_scope_kind() {
        assert_eq!(grantable_roles(None).len(), GRANTABLE_ROLES.len());
        let project = grantable_roles(Some(ScopeKind::Project));
        assert_eq!(project.len(), 4);
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
    fn grant_carries_the_role_it_confers() {
        let grant = Grant::new(
            Principal::User(UserId::new("u1")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            Scope::Organization(OrganizationId::new("o1")),
        );
        assert_eq!(grant.role.as_str(), ORGANIZATION_ADMIN_ROLE);
    }

    #[tokio::test]
    async fn anti_escalation_blocks_granting_more_than_you_hold() {
        let org = Scope::Organization(OrganizationId::new("o1"));
        // Bob holds a narrow role conferring only `manageOrgGrants` on the org;
        // Carol holds the full-control organization-admin role there.
        let roles = vec![
            test_role(
                ORGANIZATION_ADMIN_ROLE,
                ScopeKind::Organization,
                &[FULL_CONTROL],
            ),
            test_role(
                "grant-manager",
                ScopeKind::Organization,
                &["manageOrgGrants"],
            ),
        ];
        let uc = use_cases_with(
            vec![
                Grant::new(
                    Principal::User(UserId::new("bob")),
                    RoleName::new("grant-manager").unwrap(),
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

        // Bob CAN delegate the narrow role he himself holds.
        assert!(
            uc.grant(
                &bob,
                &Grant::new(
                    Principal::User(UserId::new("alice")),
                    RoleName::new("grant-manager").unwrap(),
                    org.clone(),
                ),
            )
            .await
            .is_ok(),
            "delegating a role whose permissions you hold is allowed",
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
        let mut custom = test_role(
            "01customrole",
            ScopeKind::Organization,
            &["readOrganization"],
        );
        custom.builtin = false;
        custom.key = None;
        let admin = test_role(
            ORGANIZATION_ADMIN_ROLE,
            ScopeKind::Organization,
            &[FULL_CONTROL],
        );
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

    fn owner_grant(principal: Principal, scope: Scope) -> Grant {
        Grant::new(
            principal,
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            scope,
        )
    }

    #[test]
    fn removal_orphans_scope_blocks_the_sole_human_owner() {
        let org = Scope::Organization(OrganizationId::new("o1"));
        let victim = Principal::User(UserId::new("u1"));
        let grants = vec![owner_grant(victim.clone(), org.clone())];
        assert!(
            removal_orphans_scope(&grants, &org, &victim),
            "removing the only human owner must be reported as orphaning"
        );
    }

    #[test]
    fn removal_orphans_scope_allows_when_another_human_owner_remains() {
        let org = Scope::Organization(OrganizationId::new("o1"));
        let victim = Principal::User(UserId::new("u1"));
        let grants = vec![
            owner_grant(victim.clone(), org.clone()),
            owner_grant(Principal::User(UserId::new("u2")), org.clone()),
        ];
        assert!(
            !removal_orphans_scope(&grants, &org, &victim),
            "a co-owner keeps the scope owned"
        );
    }

    #[test]
    fn removal_orphans_scope_ignores_non_owner_members() {
        let org = Scope::Organization(OrganizationId::new("o1"));
        let victim = Principal::User(UserId::new("u1"));
        // The victim holds a non-owner role, so removing them orphans nothing.
        let grants = vec![Grant::new(
            victim.clone(),
            RoleName::new(ORGANIZATION_AGENT_ROLE).unwrap(),
            org.clone(),
        )];
        assert!(
            !removal_orphans_scope(&grants, &org, &victim),
            "removing a non-owner never orphans the scope"
        );
    }

    #[test]
    fn removal_orphans_scope_does_not_count_app_owners_as_human() {
        let org = Scope::Organization(OrganizationId::new("o1"));
        let victim = Principal::User(UserId::new("u1"));
        // Another owner exists but it is an App, which must not count as the
        // retained human owner (mirrors revoke's last-human-owner rule).
        let grants = vec![
            owner_grant(victim.clone(), org.clone()),
            owner_grant(Principal::App(AppId::new("agent-1")), org.clone()),
        ];
        assert!(
            removal_orphans_scope(&grants, &org, &victim),
            "an App owner is not a human owner"
        );
    }

    #[test]
    fn removal_orphans_scope_is_scoped() {
        let org = Scope::Organization(OrganizationId::new("o1"));
        let other = Scope::Organization(OrganizationId::new("o2"));
        let victim = Principal::User(UserId::new("u1"));
        // The victim owns a *different* org; removing them from o1 (where they
        // own nothing) does not orphan it.
        let grants = vec![owner_grant(victim.clone(), other)];
        assert!(!removal_orphans_scope(&grants, &org, &victim));
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

    #[tokio::test]
    async fn a_project_grant_requires_admission_to_the_organization() {
        // The tenant boundary. Admitting someone needs `manageOrgGrants`; a
        // project admin then distributes access among the people already
        // admitted, and cannot pull in an arbitrary account.
        let roles = vec![
            test_role(PROJECT_ADMIN_ROLE, ScopeKind::Project, &[FULL_CONTROL]),
            test_role(
                ORGANIZATION_MEMBER_ROLE,
                ScopeKind::Organization,
                &["readOrganization"],
            ),
        ];
        let service = CallerContext::Service(ServiceIdentity::recorder());
        let alice = Principal::User(UserId::new("alice"));
        let grant = Grant::new(
            alice.clone(),
            RoleName::new(PROJECT_ADMIN_ROLE).unwrap(),
            Scope::Project(ProjectId::new("p1")),
        );

        // Alice holds nothing under o1 yet.
        let uc = use_cases_with(
            vec![],
            roles.clone(),
            Arc::new(RecordingRegistry::default()),
        );
        assert!(
            uc.grant(&service, &grant).await.is_err(),
            "a project grant to someone the organization has not admitted must be refused"
        );

        // Once she holds the floor role on the org, the project grant lands.
        let admitted = Grant::new(
            alice,
            RoleName::new(ORGANIZATION_MEMBER_ROLE).unwrap(),
            Scope::Organization(OrganizationId::new("o1")),
        );
        let uc = use_cases_with(
            vec![admitted],
            roles,
            Arc::new(RecordingRegistry::default()),
        );
        assert!(
            uc.grant(&service, &grant).await.is_ok(),
            "a project grant to an admitted user is accepted"
        );
    }

    #[tokio::test]
    async fn an_organization_grant_needs_no_prior_admission() {
        // Otherwise nobody could ever join: the org-scoped grant *is* the
        // admission.
        let roles = vec![test_role(
            ORGANIZATION_ADMIN_ROLE,
            ScopeKind::Organization,
            &[FULL_CONTROL],
        )];
        let uc = use_cases_with(vec![], roles, Arc::new(RecordingRegistry::default()));
        let grant = Grant::new(
            Principal::User(UserId::new("alice")),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            Scope::Organization(OrganizationId::new("o1")),
        );
        assert!(
            uc.grant(&CallerContext::Service(ServiceIdentity::recorder()), &grant)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn app_grants_need_no_admission() {
        // Machine Apps are owned by an organization by construction, never
        // admitted to one.
        let roles = vec![test_role(
            PROJECT_AGENT_ROLE,
            ScopeKind::Project,
            &["readPipeline", "executeJob"],
        )];
        let uc = use_cases_with(vec![], roles, Arc::new(RecordingRegistry::default()));
        let grant = Grant::new(
            Principal::App(AppId::new("agent-1")),
            RoleName::new(PROJECT_AGENT_ROLE).unwrap(),
            Scope::Project(ProjectId::new("p1")),
        );
        assert!(
            uc.grant(&CallerContext::Service(ServiceIdentity::recorder()), &grant)
                .await
                .is_ok(),
            "an App grant needs no admission"
        );
    }

    #[tokio::test]
    async fn revoke_all_access_refuses_to_strip_the_last_owner() {
        // The kill switch obeys the same rule as a single revoke: a scope must
        // never be left without a human owner.
        let org = Scope::Organization(OrganizationId::new("o1"));
        let alice = Principal::User(UserId::new("alice"));
        let sole_owner = Grant::new(
            alice.clone(),
            RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
            org.clone(),
        );
        let uc = use_cases(vec![sole_owner], Arc::new(RecordingRegistry::default()));
        let service = CallerContext::Service(ServiceIdentity::recorder());

        assert!(
            uc.revoke_all_access(&service, &alice, &org).await.is_err(),
            "stripping the only owner of an organization must be refused"
        );
    }

    #[tokio::test]
    async fn revoke_all_access_disconnects_a_machine_principal() {
        // An App that just lost its access must not keep running on an
        // already-open stream.
        let app = Principal::App(AppId::new("agent-1"));
        let project = Scope::Project(ProjectId::new("p1"));
        let reg = Arc::new(RecordingRegistry::default());
        let uc = use_cases(vec![], reg.clone());

        uc.revoke_all_access(
            &CallerContext::Service(ServiceIdentity::recorder()),
            &app,
            &project,
        )
        .await
        .expect("revoke all");

        assert_eq!(reg.disconnected(), vec!["agent-1".to_string()]);
    }
}
