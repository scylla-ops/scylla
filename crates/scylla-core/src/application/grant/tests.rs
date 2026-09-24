//! The grant's actions through the engine, on stub ports.

use super::*;
use crate::domain::caller::ServiceIdentity;
use crate::domain::ids::{AppId, OrganizationId, ProjectId, UserId};
use crate::domain::permission::ResourceRef;
use crate::domain::role::RoleName;
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService, actions};
use crate::test_support::stubs::{CountingPolicy, StubGrants, StubRegistry, StubRoles};
use async_trait::async_trait;
use scylla_auth::authz::*;
use scylla_auth::authz::{ResourceAncestors, Role};
use scylla_extension::Actions;

struct StubAncestry;
#[async_trait]
impl AuthzEntityProvider for StubAncestry {
    async fn resource_ancestors(&self, _resource: &ResourceRef) -> DomainResult<ResourceAncestors> {
        Ok(ResourceAncestors {
            organization: Some(OrganizationId::new("o1")),
            ..Default::default()
        })
    }
    async fn app_is_active(&self, _app: &AppId) -> DomainResult<bool> {
        Ok(true)
    }
}

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

struct Lab {
    actions: Actions,
    uc: GrantUseCases,
    grants: Arc<StubGrants>,
    registry: Arc<StubRegistry>,
}

impl Lab {
    async fn grant(&self, caller: &CallerContext, grant: &Grant) -> DomainResult<Grant> {
        let cmd = CreateGrant {
            principal: grant.principal.clone(),
            role: grant.role.clone(),
            scope: grant.scope.clone(),
        };
        self.actions.run(&self.uc, caller, cmd).await
    }

    async fn revoke_all_access(
        &self,
        caller: &CallerContext,
        principal: &Principal,
        scope: &Scope,
    ) -> DomainResult<u64> {
        let cmd = RevokeAllAccess {
            principal: principal.clone(),
            scope: scope.clone(),
        };
        self.actions.run(&self.uc, caller, cmd).await
    }

    async fn list(&self, scope: Option<Scope>) -> DomainResult<Vec<Grant>> {
        self.actions
            .run(&self.uc, &admin(), ListGrants { scope })
            .await
    }
}

fn lab(grants: Vec<Grant>) -> Lab {
    lab_with(grants, vec![], Arc::new(RecordingPermissionService::new()))
}

fn lab_with(grants: Vec<Grant>, roles: Vec<Role>, permissions: Arc<dyn PermissionService>) -> Lab {
    let grants = Arc::new(StubGrants::new(grants));
    let registry = Arc::new(StubRegistry::default());
    Lab {
        actions: actions(permissions.clone()),
        uc: GrantUseCases::new(
            grants.clone(),
            Arc::new(StubRoles::new(roles)),
            Arc::new(CountingPolicy::default()),
            permissions,
            registry.clone(),
            Arc::new(StubAncestry),
        ),
        grants,
        registry,
    }
}

fn admin() -> CallerContext {
    CallerContext::User(UserId::new("admin"))
}

fn service() -> CallerContext {
    CallerContext::Service(ServiceIdentity::recorder())
}

fn org() -> Scope {
    Scope::Organization(OrganizationId::new("o1"))
}

fn owner_grant(principal: Principal, scope: Scope) -> Grant {
    Grant::new(
        principal,
        RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
        scope,
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
        org(),
    );
    assert_eq!(grant.role.as_str(), ORGANIZATION_ADMIN_ROLE);
}

#[tokio::test]
async fn a_create_checks_the_permission_of_its_scope_then_stores_the_grant() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab_with(
        vec![],
        vec![test_role(
            ORGANIZATION_ADMIN_ROLE,
            ScopeKind::Organization,
            &[FULL_CONTROL],
        )],
        permissions.clone(),
    );
    let grant = owner_grant(Principal::User(UserId::new("alice")), org());

    let stored = lab.grant(&service(), &grant).await.unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::ManageOrgGrants(OrganizationId::new("o1"))]
    );
    assert_eq!(stored.principal, grant.principal);
    assert_eq!(stored.scope, grant.scope);
    assert_eq!(lab.grants.created().as_slice(), [stored]);
}

#[tokio::test]
async fn a_denied_create_never_stores() {
    let lab = lab_with(
        vec![],
        vec![test_role(
            ORGANIZATION_ADMIN_ROLE,
            ScopeKind::Organization,
            &[FULL_CONTROL],
        )],
        Arc::new(DenyingPermissionService::new()),
    );
    let grant = owner_grant(Principal::User(UserId::new("alice")), org());

    let err = lab.grant(&service(), &grant).await.unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.grants.created().is_empty());
}

#[tokio::test]
async fn anti_escalation_blocks_granting_more_than_you_hold() {
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
    let lab = lab_with(
        vec![
            Grant::new(
                Principal::User(UserId::new("bob")),
                RoleName::new("grant-manager").unwrap(),
                org(),
            ),
            owner_grant(Principal::User(UserId::new("carol")), org()),
        ],
        roles,
        Arc::new(RecordingPermissionService::new()),
    );
    let bob = CallerContext::User(UserId::new("bob"));
    let carol = CallerContext::User(UserId::new("carol"));

    assert!(
        lab.grant(
            &bob,
            &owner_grant(Principal::User(UserId::new("alice")), org())
        )
        .await
        .is_err(),
        "escalation to full control must be blocked",
    );

    assert!(
        lab.grant(
            &bob,
            &Grant::new(
                Principal::User(UserId::new("alice")),
                RoleName::new("grant-manager").unwrap(),
                org(),
            ),
        )
        .await
        .is_ok(),
        "delegating a role whose permissions you hold is allowed",
    );

    assert!(
        lab.grant(
            &carol,
            &owner_grant(Principal::User(UserId::new("dave")), org())
        )
        .await
        .is_ok(),
        "a full-control admin may grant",
    );
}

#[tokio::test]
async fn custom_role_grantable_via_db_with_scope_check() {
    let mut custom = test_role(
        "01customrole",
        ScopeKind::Organization,
        &["readOrganization"],
    );
    custom.builtin = false;
    custom.key = None;
    let admin_role = test_role(
        ORGANIZATION_ADMIN_ROLE,
        ScopeKind::Organization,
        &[FULL_CONTROL],
    );
    let lab = lab_with(
        vec![owner_grant(Principal::User(UserId::new("owner")), org())],
        vec![custom, admin_role],
        Arc::new(RecordingPermissionService::new()),
    );
    let owner = CallerContext::User(UserId::new("owner"));

    assert!(
        lab.grant(
            &owner,
            &Grant::new(
                Principal::User(UserId::new("alice")),
                RoleName::new("01customrole").unwrap(),
                org(),
            ),
        )
        .await
        .is_ok(),
        "a custom role valid at its scope must be grantable",
    );

    assert!(
        lab.grant(
            &owner,
            &Grant::new(
                Principal::User(UserId::new("alice")),
                RoleName::new("ghost").unwrap(),
                org(),
            ),
        )
        .await
        .is_err(),
        "unknown role must be rejected",
    );

    assert!(
        lab.grant(
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
async fn a_project_grant_requires_admission_to_the_organization() {
    let roles = vec![
        test_role(PROJECT_ADMIN_ROLE, ScopeKind::Project, &[FULL_CONTROL]),
        test_role(
            ORGANIZATION_MEMBER_ROLE,
            ScopeKind::Organization,
            &["readOrganization"],
        ),
    ];
    let alice = Principal::User(UserId::new("alice"));
    let grant = Grant::new(
        alice.clone(),
        RoleName::new(PROJECT_ADMIN_ROLE).unwrap(),
        Scope::Project(ProjectId::new("p1")),
    );

    let lab = lab_with(
        vec![],
        roles.clone(),
        Arc::new(RecordingPermissionService::new()),
    );
    assert!(
        lab.grant(&service(), &grant).await.is_err(),
        "a project grant to someone the organization has not admitted must be refused"
    );

    let admitted = Grant::new(
        alice,
        RoleName::new(ORGANIZATION_MEMBER_ROLE).unwrap(),
        org(),
    );
    let lab = lab_with(
        vec![admitted],
        roles,
        Arc::new(RecordingPermissionService::new()),
    );
    assert!(
        lab.grant(&service(), &grant).await.is_ok(),
        "a project grant to an admitted user is accepted"
    );
}

#[tokio::test]
async fn an_organization_grant_needs_no_prior_admission() {
    let roles = vec![test_role(
        ORGANIZATION_ADMIN_ROLE,
        ScopeKind::Organization,
        &[FULL_CONTROL],
    )];
    let lab = lab_with(vec![], roles, Arc::new(RecordingPermissionService::new()));
    let grant = owner_grant(Principal::User(UserId::new("alice")), org());
    assert!(lab.grant(&service(), &grant).await.is_ok());
}

#[tokio::test]
async fn app_grants_need_no_admission() {
    let roles = vec![test_role(
        PROJECT_AGENT_ROLE,
        ScopeKind::Project,
        &["readPipeline", "executeJob"],
    )];
    let lab = lab_with(vec![], roles, Arc::new(RecordingPermissionService::new()));
    let grant = Grant::new(
        Principal::App(AppId::new("agent-1")),
        RoleName::new(PROJECT_AGENT_ROLE).unwrap(),
        Scope::Project(ProjectId::new("p1")),
    );
    assert!(
        lab.grant(&service(), &grant).await.is_ok(),
        "an App grant needs no admission"
    );
}

#[tokio::test]
async fn a_list_without_a_scope_asks_for_the_system_permission_and_returns_every_grant() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let other = Scope::Organization(OrganizationId::new("o2"));
    let lab = lab_with(
        vec![
            owner_grant(Principal::User(UserId::new("u1")), org()),
            owner_grant(Principal::User(UserId::new("u2")), other),
        ],
        vec![],
        permissions.clone(),
    );

    let grants = lab.list(None).await.unwrap();

    assert_eq!(grants.len(), 2);
    assert_eq!(
        permissions.permissions(),
        vec![Permission::ManageSystemGrants]
    );
}

#[tokio::test]
async fn a_list_with_a_scope_asks_for_that_scope_and_keeps_its_grants() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let other = Scope::Organization(OrganizationId::new("o2"));
    let lab = lab_with(
        vec![
            owner_grant(Principal::User(UserId::new("u1")), org()),
            owner_grant(Principal::User(UserId::new("u2")), other),
        ],
        vec![],
        permissions.clone(),
    );

    let grants = lab.list(Some(org())).await.unwrap();

    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0].scope, org());
    assert_eq!(
        permissions.permissions(),
        vec![Permission::ManageOrgGrants(OrganizationId::new("o1"))]
    );
}

#[tokio::test]
async fn revoking_app_grant_disconnects_the_agent() {
    let grant = Grant::new(
        Principal::App(AppId::new("agent-1")),
        RoleName::new(ORGANIZATION_AGENT_ROLE).unwrap(),
        org(),
    );
    let lab = lab(vec![grant.clone()]);

    lab.uc.revoke(&admin(), &grant.id).await.unwrap();

    assert_eq!(lab.registry.disconnected(), vec![AppId::new("agent-1")]);
}

#[tokio::test]
async fn cannot_revoke_last_owner_of_scope() {
    let grant = owner_grant(Principal::User(UserId::new("u1")), org());
    let lab = lab(vec![grant.clone()]);

    assert!(
        lab.uc.revoke(&admin(), &grant.id).await.is_err(),
        "revoking the last owner must be blocked"
    );
}

#[tokio::test]
async fn can_revoke_owner_when_another_exists() {
    let g1 = owner_grant(Principal::User(UserId::new("u1")), org());
    let g2 = owner_grant(Principal::User(UserId::new("u2")), org());
    let lab = lab(vec![g1.clone(), g2]);

    assert!(
        lab.uc.revoke(&admin(), &g1.id).await.is_ok(),
        "revoking one of two owners is allowed"
    );
}

#[tokio::test]
async fn revoking_user_grant_leaves_agents_alone() {
    let grant = owner_grant(Principal::User(UserId::new("u1")), org());
    let co_owner = owner_grant(Principal::User(UserId::new("u2")), org());
    let lab = lab(vec![grant.clone(), co_owner]);

    lab.uc.revoke(&admin(), &grant.id).await.unwrap();

    assert!(lab.registry.disconnected().is_empty());
}

#[tokio::test]
async fn a_revoke_checks_the_permission_on_the_loaded_grant_scope() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let grant = Grant::new(
        Principal::App(AppId::new("agent-1")),
        RoleName::new(ORGANIZATION_AGENT_ROLE).unwrap(),
        org(),
    );
    let lab = lab_with(vec![grant.clone()], vec![], permissions.clone());

    lab.uc.revoke(&admin(), &grant.id).await.unwrap();
    lab.uc.revoke(&admin(), "unknown").await.unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![
            Permission::ManageOrgGrants(OrganizationId::new("o1")),
            Permission::ManageSystemGrants,
        ]
    );
}

#[test]
fn removal_orphans_scope_blocks_the_sole_human_owner() {
    let victim = Principal::User(UserId::new("u1"));
    let grants = vec![owner_grant(victim.clone(), org())];
    assert!(
        removal_orphans_scope(&grants, &org(), &victim),
        "removing the only human owner must be reported as orphaning"
    );
}

#[test]
fn removal_orphans_scope_allows_when_another_human_owner_remains() {
    let victim = Principal::User(UserId::new("u1"));
    let grants = vec![
        owner_grant(victim.clone(), org()),
        owner_grant(Principal::User(UserId::new("u2")), org()),
    ];
    assert!(
        !removal_orphans_scope(&grants, &org(), &victim),
        "a co-owner keeps the scope owned"
    );
}

#[test]
fn removal_orphans_scope_ignores_non_owner_members() {
    let victim = Principal::User(UserId::new("u1"));
    let grants = vec![Grant::new(
        victim.clone(),
        RoleName::new(ORGANIZATION_AGENT_ROLE).unwrap(),
        org(),
    )];
    assert!(
        !removal_orphans_scope(&grants, &org(), &victim),
        "removing a non-owner never orphans the scope"
    );
}

#[test]
fn removal_orphans_scope_does_not_count_app_owners_as_human() {
    let victim = Principal::User(UserId::new("u1"));
    let grants = vec![
        owner_grant(victim.clone(), org()),
        owner_grant(Principal::App(AppId::new("agent-1")), org()),
    ];
    assert!(
        removal_orphans_scope(&grants, &org(), &victim),
        "an App owner is not a human owner"
    );
}

#[test]
fn removal_orphans_scope_is_scoped() {
    let other = Scope::Organization(OrganizationId::new("o2"));
    let victim = Principal::User(UserId::new("u1"));
    let grants = vec![owner_grant(victim.clone(), other)];
    assert!(!removal_orphans_scope(&grants, &org(), &victim));
}

#[tokio::test]
async fn revoke_all_access_refuses_to_strip_the_last_owner() {
    let alice = Principal::User(UserId::new("alice"));
    let lab = lab(vec![owner_grant(alice.clone(), org())]);

    assert!(
        lab.revoke_all_access(&service(), &alice, &org())
            .await
            .is_err(),
        "stripping the only owner of an organization must be refused"
    );
    assert!(lab.registry.disconnected().is_empty());
}

#[tokio::test]
async fn revoke_all_access_checks_the_scope_permission_and_disconnects_a_machine_principal() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let app = Principal::App(AppId::new("agent-1"));
    let project = Scope::Project(ProjectId::new("p1"));
    let lab = lab_with(vec![], vec![], permissions.clone());

    lab.revoke_all_access(&service(), &app, &project)
        .await
        .expect("revoke all");

    assert_eq!(
        permissions.permissions(),
        vec![Permission::ManageProjectGrants(ProjectId::new("p1"))]
    );
    assert_eq!(lab.registry.disconnected(), vec![AppId::new("agent-1")]);
}
