use super::cedar_authz::{euid, parent_set, principal_parts, resource_parts, resource_uid};
use crate::application::PermissionService;
use crate::application::audit::{AuditDecision, AuditEntry, AuditLog};
use crate::application::authz::entity_provider::AuthzEntityProvider;
use crate::application::authz::grant::{Grant, GrantRepository, Principal, Scope};
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::role::{Role, RoleRepository};
use crate::application::authz::visibility::{
    Visibility, VisibilityResolver, visibility_from_grants,
};
use crate::application::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::{Permission, ResourceRef};
use async_trait::async_trait;
use cedar_policy::{
    Authorizer, Context, Decision, Entities, Entity, EntityUid, PolicyId, PolicySet, Request,
    Schema, SlotId, Template, ValidationMode, Validator,
};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tracing::{info, instrument, warn};

const SCHEMA_SRC: &str = include_str!("cedar/schema.cedarschema");
const POLICIES_SRC: &str = include_str!("cedar/policies.cedar");

/// The Cedar permit body for a role, instantiated per grant via the `?principal`
/// / `?resource` slots. A full-control role (`*` permission set) gets an
/// unconstrained action; any other role lists its permission keys explicitly.
/// The body carries no condition: holding the grant *is* the authority, and
/// `resource in ?resource` is what bounds it. Returns `None` for a role that
/// confers nothing (empty permission set): it gets no template, so a grant of it
/// links to nothing (and is logged as unlinkable).
fn role_template_src(role: &Role) -> Option<String> {
    let action = if role.is_full_control() {
        "action".to_string()
    } else {
        let actions: Vec<String> = role
            .permissions
            .iter()
            .map(|key| format!("        Scylla::Action::\"{key}\""))
            .collect();
        if actions.is_empty() {
            return None;
        }
        format!("action in [\n{}\n    ]", actions.join(",\n"))
    };
    Some(format!(
        "permit (\n    principal == ?principal,\n    {action},\n    resource in ?resource\n);\n"
    ))
}

/// Cedar-backed authorization. Static policies + schema are compiled into the
/// binary and validated at construction; per-role templates are generated from
/// the role store and each stored grant links an instance of its role's
/// template. The
/// principal's org/project memberships, plus the resource's ancestor chain, are
/// materialised per request via the [`AuthzEntityProvider`].
pub struct CedarPermissionService<EP: AuthzEntityProvider> {
    /// Live policy set behind an `RwLock<Arc<…>>` so it can be swapped atomically
    /// on reload. The new set is built fully off-lock; the write lock is only held
    /// for the pointer swap, so reads never block on a rebuild.
    policies: RwLock<Arc<PolicySet>>,
    authorizer: Authorizer,
    entity_provider: Arc<EP>,
    /// Stores backing the live set; re-read on every `reload` to rebuild it.
    /// Roles supply the per-role template bodies; grants link instances of them.
    role_repo: Arc<dyn RoleRepository>,
    grant_repo: Arc<dyn GrantRepository>,
    audit: Arc<dyn AuditLog>,
}

impl<EP: AuthzEntityProvider> CedarPermissionService<EP> {
    /// Build the live policy set from the stores, then keep handles to those
    /// stores so the set can be rebuilt on demand (`reload`). Fails fast if the
    /// embedded schema/policies don't typecheck — catching drift at startup.
    pub async fn new(
        entity_provider: Arc<EP>,
        role_repo: Arc<dyn RoleRepository>,
        grant_repo: Arc<dyn GrantRepository>,
        audit: Arc<dyn AuditLog>,
    ) -> DomainResult<Self> {
        let roles = role_repo.list_all().await?;
        let grants = grant_repo.list_all().await?;
        let policies = Self::build_policy_set(&roles, &grants)?;

        Ok(Self {
            policies: RwLock::new(Arc::new(policies)),
            authorizer: Authorizer::new(),
            entity_provider,
            role_repo,
            grant_repo,
            audit,
        })
    }

    /// Assemble a complete policy set: static base + scoped-role templates,
    /// validated against the schema, then the stored grants linked as template
    /// instances. Shared by `new` and `reload`; on any error the caller keeps the
    /// previous live set.
    fn build_policy_set(roles: &[Role], grants: &[Grant]) -> DomainResult<PolicySet> {
        let (schema, _warnings) = Schema::from_cedarschema_str(SCHEMA_SRC)
            .map_err(|e| DomainError::Internal(format!("cedar schema parse: {e}")))?;

        let mut policies = PolicySet::from_str(POLICIES_SRC)
            .map_err(|e| DomainError::Internal(format!("cedar policy parse: {e}")))?;

        // Register a template per role, its action set generated from the role's
        // permissions (full control → unconstrained action; otherwise an explicit
        // list). Grants then link an instance of their role's template, matched
        // by role id. A role that confers nothing gets no template.
        for role in roles {
            let Some(src) = role_template_src(role) else {
                continue;
            };
            let template =
                Template::parse(Some(PolicyId::new(role.id.as_str())), &src).map_err(|e| {
                    DomainError::Internal(format!("cedar template parse {}: {e}", role.id))
                })?;
            policies.add_template(template).map_err(|e| {
                DomainError::Internal(format!("cedar add template {}: {e}", role.id))
            })?;
        }

        let result = Validator::new(schema).validate(&policies, ValidationMode::Strict);
        if !result.validation_passed() {
            let errs: Vec<String> = result
                .validation_errors()
                .map(ToString::to_string)
                .collect();
            return Err(DomainError::Internal(format!(
                "cedar policy validation failed: {errs:?}"
            )));
        }

        // Grants are instances of an already-validated template, so they are
        // linked after validation (matching original boot behaviour).
        for grant in grants {
            if let Err(e) = Self::link_grant(&mut policies, grant) {
                warn!(grant_id = %grant.id, error = %e, "skipping unlinkable grant");
            }
        }

        Ok(policies)
    }

    /// Emit a stored grant into the policy set: link an instance of its role's
    /// template (template id == role id; `?principal` → the principal,
    /// `?resource` → the granted scope), keyed `grant-<id>`.
    fn link_grant(policies: &mut PolicySet, grant: &Grant) -> DomainResult<()> {
        let principal_uid = match &grant.principal {
            Principal::User(id) => euid("Scylla::User", id.as_str())?,
            Principal::App(id) => euid("Scylla::App", id.as_str())?,
        };
        let resource_uid = match &grant.scope {
            // System is the tenancy root; a grant here (e.g. system-admin) covers
            // everything beneath via `resource in ?resource`.
            Scope::System => euid("Scylla::System", "root")?,
            Scope::Organization(id) => euid("Scylla::Organization", id.as_str())?,
            Scope::Project(id) => euid("Scylla::Project", id.as_str())?,
        };
        let vals = HashMap::from([
            (SlotId::principal(), principal_uid),
            (SlotId::resource(), resource_uid),
        ]);
        policies
            .link(
                PolicyId::new(grant.role.as_str()),
                PolicyId::new(format!("grant-{}", grant.id)),
                vals,
            )
            .map_err(|e| DomainError::Internal(format!("cedar link: {e}")))
    }

    /// Build the principal entity (+ role entities) for the caller. Returns the
    /// principal UID and every entity that must be in the store for it.
    async fn principal_entities(
        &self,
        caller: &CallerContext,
    ) -> DomainResult<(EntityUid, Vec<Entity>)> {
        match caller {
            CallerContext::User(id) => {
                // Like every principal, a user carries no attributes: what it may
                // do lives in the grants linked into the policy set, never on the
                // entity, so nothing has to be materialised per request.
                let uid = euid("Scylla::User", id.as_str())?;
                Ok((uid.clone(), vec![Entity::new_no_attrs(uid, HashSet::new())]))
            }
            CallerContext::App(id) => {
                // A machine principal carries no roles/ABAC attrs of its own; its
                // access comes entirely from linked scoped grants (agent role).
                let uid = euid("Scylla::App", id.as_str())?;
                Ok((uid.clone(), vec![Entity::new_no_attrs(uid, HashSet::new())]))
            }
            CallerContext::Service(svc) => {
                let uid = euid("Scylla::Service", svc.as_str())?;
                Ok((uid.clone(), vec![Entity::new_no_attrs(uid, HashSet::new())]))
            }
            CallerContext::Anonymous => Err(DomainError::Forbidden(
                "Anonymous caller is not permitted".to_string(),
            )),
        }
    }

    /// Build the resource entity and its ancestor chain so Cedar `in` checks
    /// (RBAC scope + ABAC membership) resolve through the tenancy hierarchy.
    async fn resource_entities(
        &self,
        resource: &ResourceRef,
    ) -> DomainResult<(EntityUid, Vec<Entity>)> {
        let uid = resource_uid(resource)?;
        let ancestors = self.entity_provider.resource_ancestors(resource).await?;

        // System is the tenancy root: every Organization is `in` System, so a
        // System-scoped grant (system-admin) reaches the whole tree.
        let system_uid = euid("Scylla::System", "root")?;

        let org_uid = ancestors
            .organization
            .as_ref()
            .map(|o| euid("Scylla::Organization", o.as_str()))
            .transpose()?;
        let project_uid = ancestors
            .project
            .as_ref()
            .map(|p| euid("Scylla::Project", p.as_str()))
            .transpose()?;
        let pipeline_uid = ancestors
            .pipeline
            .as_ref()
            .map(|p| euid("Scylla::Pipeline", p.as_str()))
            .transpose()?;

        let mut entities = Vec::new();
        // The System root entity (parent of every organization).
        entities.push(Entity::new_no_attrs(system_uid.clone(), HashSet::new()));
        if let Some(o) = &org_uid {
            entities.push(Entity::new_no_attrs(
                o.clone(),
                parent_set(Some(&system_uid)),
            ));
        }
        if let Some(p) = &project_uid {
            entities.push(Entity::new_no_attrs(
                p.clone(),
                parent_set(org_uid.as_ref()),
            ));
        }
        if let Some(pl) = &pipeline_uid {
            entities.push(Entity::new_no_attrs(
                pl.clone(),
                parent_set(project_uid.as_ref()),
            ));
        }

        // Direct parent of the resource leaf, deepest available level.
        let leaf_parent = match resource {
            ResourceRef::Job(_) => pipeline_uid.as_ref(),
            ResourceRef::Pipeline(_) => project_uid.as_ref(),
            ResourceRef::Project(_) | ResourceRef::App(_) => org_uid.as_ref(),
            // An organization's parent is the System root.
            ResourceRef::Organization(_) => Some(&system_uid),
            // So is a user's. This used to be supplied by `user_entity`; without
            // it a System-scoped grant stops reaching user-targeted actions, and
            // no test catches it because self-access comes from the static
            // `self` policy instead.
            ResourceRef::User(_) => Some(&system_uid),
            _ => None,
        };

        // The leaf entity itself. No entity in the schema carries attributes.
        let leaf = Entity::new_no_attrs(uid.clone(), parent_set(leaf_parent));
        entities.push(leaf);

        Ok((uid, entities))
    }

    /// Emit both audit trails (the live `audit` tracing target + the persistent
    /// store) for a single authorization verdict. Shared by the Cedar decision
    /// path and the principal-liveness gate so every deny is recorded the same
    /// way.
    fn record_decision(
        &self,
        caller: &CallerContext,
        perm: &Permission,
        resource: &ResourceRef,
        decision: AuditDecision,
        reason: Option<String>,
        policies: Vec<String>,
    ) {
        let (principal_kind, principal_id) = principal_parts(caller);
        let (resource_kind, resource_id) = resource_parts(resource);

        match decision {
            AuditDecision::Allow => info!(
                target: "audit",
                who = %caller, action = perm.key(), resource = %resource,
                decision = "allow", policies = ?policies, "action authorized"
            ),
            AuditDecision::Deny => warn!(
                target: "audit",
                who = %caller, action = perm.key(), resource = %resource,
                decision = "deny", policies = ?policies, "action denied"
            ),
        }

        self.audit.record(AuditEntry {
            occurred_at: Utc::now(),
            principal_kind,
            principal_id,
            action: perm.key(),
            resource_kind,
            resource_id,
            decision,
            policies,
            reason,
        });
    }
}

#[async_trait]
impl<EP: AuthzEntityProvider + 'static> PermissionService for CedarPermissionService<EP> {
    #[instrument(skip(self, caller, perm), fields(caller = ?caller, action = perm.key()))]
    async fn check(&self, caller: &CallerContext, perm: Permission) -> DomainResult<()> {
        let resource = perm.resource();

        // Durable liveness gate: re-validate the principal on EVERY action, not
        // just at stream-open / token-issue time. A long-lived agent stream
        // otherwise keeps acting after its backing App is disabled or deleted.
        // This `check` is the single chokepoint every privileged operation flows
        // through, so the guarantee holds for streamed and one-shot calls alike.
        if let CallerContext::App(app_id) = caller {
            if !self.entity_provider.app_is_active(app_id).await? {
                self.record_decision(
                    caller,
                    &perm,
                    &resource,
                    AuditDecision::Deny,
                    Some("app principal is disabled or no longer exists".to_string()),
                    Vec::new(),
                );
                return Err(DomainError::Forbidden("Action denied".to_string()));
            }
        }

        let (principal_uid, principal_entities) = self.principal_entities(caller).await?;
        let (resource_uid, resource_entities) = self.resource_entities(&resource).await?;
        let action_uid = euid("Scylla::Action", perm.key())?;

        // Dedup by UID (e.g. a user reading itself: principal == resource).
        // Principal entities win so the principal keeps its roles + attrs.
        let mut by_uid: HashMap<String, Entity> = HashMap::new();
        for e in resource_entities.into_iter().chain(principal_entities) {
            by_uid.insert(e.uid().to_string(), e);
        }
        let entities = Entities::from_entities(by_uid.into_values(), None)
            .map_err(|e| DomainError::Internal(format!("cedar entities: {e}")))?;

        let request = Request::new(
            principal_uid,
            action_uid,
            resource_uid,
            Context::empty(),
            None,
        )
        .map_err(|e| DomainError::Internal(format!("cedar request: {e}")))?;

        // Snapshot the live set (cheap Arc clone) and release the lock before
        // evaluating, so a concurrent reload never blocks a check.
        let policies = self
            .policies
            .read()
            .expect("policy set lock poisoned")
            .clone();
        let response = self
            .authorizer
            .is_authorized(&request, &policies, &entities);

        // The Cedar policy ids that determined the verdict (admin/ABAC rule or a
        // linked grant) — captured for the audit trail.
        let policies: Vec<String> = response
            .diagnostics()
            .reason()
            .map(ToString::to_string)
            .collect();
        let (audit_decision, reason, result) = match response.decision() {
            Decision::Allow => (AuditDecision::Allow, None, Ok(())),
            Decision::Deny => {
                let errors: Vec<String> = response
                    .diagnostics()
                    .errors()
                    .map(ToString::to_string)
                    .collect();
                let reason = (!errors.is_empty()).then(|| errors.join("; "));
                (
                    AuditDecision::Deny,
                    reason,
                    Err(DomainError::Forbidden("Action denied".to_string())),
                )
            }
        };

        self.record_decision(caller, &perm, &resource, audit_decision, reason, policies);
        result
    }
}

#[async_trait]
impl<EP: AuthzEntityProvider + 'static> VisibilityResolver for CedarPermissionService<EP> {
    /// Read the same stores the policy set is built from, and fold the caller's
    /// grants into the scopes where they hold `permission_key`. Deliberately not
    /// a Cedar query: Cedar answers "may this principal do X to that entity",
    /// one entity at a time, which cannot express a filter over a page.
    ///
    /// The two paths agree because they read the same rows. A grant only widens
    /// visibility here if its role confers the permission, which is the same
    /// condition that puts the action in the role's generated template.
    #[instrument(skip(self, caller))]
    async fn visible_scopes(
        &self,
        caller: &CallerContext,
        permission_key: &str,
    ) -> DomainResult<Visibility> {
        // A service acts as the system; an App or User is filtered by its
        // grants; Anonymous never reaches a listing.
        let principal = match caller {
            CallerContext::Service(_) => return Ok(Visibility::All),
            CallerContext::Anonymous => return Ok(Visibility::none()),
            _ => match Principal::from_caller(caller) {
                Some(p) => p,
                None => return Ok(Visibility::none()),
            },
        };

        let role_permissions: HashMap<String, Vec<String>> = self
            .role_repo
            .list_all()
            .await?
            .into_iter()
            .map(|r| (r.id, r.permissions))
            .collect();
        let grants = self.grant_repo.list_all().await?;

        Ok(visibility_from_grants(
            &role_permissions,
            &grants,
            &principal,
            permission_key,
        ))
    }
}

#[async_trait]
impl<EP: AuthzEntityProvider + 'static> PolicyControl for CedarPermissionService<EP> {
    /// Rebuild the live policy set from the stores and swap it in atomically.
    /// On failure the previous set is kept, so a check is never served by a
    /// broken or partial set.
    #[instrument(skip(self))]
    async fn reload(&self) -> DomainResult<()> {
        let roles = self.role_repo.list_all().await?;
        let grants = self.grant_repo.list_all().await?;
        let policies = Self::build_policy_set(&roles, &grants)?;
        *self.policies.write().expect("policy set lock poisoned") = Arc::new(policies);
        info!(target: "audit", "authorization policy set reloaded");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::authz::entity_provider::ResourceAncestors;
    use crate::application::authz::grant::{
        ORGANIZATION_ADMIN_ROLE, ORGANIZATION_AGENT_ROLE, ORGANIZATION_TRIGGER_RUNNER_ROLE,
        ORGANIZATION_VIEWER_ROLE, PROJECT_ADMIN_ROLE, PROJECT_AGENT_ROLE, PROJECT_DEVELOPER_ROLE,
        SYSTEM_ADMIN_ROLE, ScopeKind,
    };
    use crate::application::authz::role::FULL_CONTROL;
    use crate::application::caller::ServiceIdentity;
    use crate::domain::entities::{AppId, OrganizationId, PipelineId, ProjectId, UserId};
    use crate::domain::value_objects::role::RoleName;

    /// The five builtin roles as the seed migration defines them: admin roles
    /// confer full control (`*`), agent roles the four job-execution
    /// permissions. The Cedar templates are generated from these, so the stub
    /// must match the seed for the admin/agent behaviour tests to hold.
    fn builtin_roles() -> Vec<Role> {
        let admin = |id: &str, scope: ScopeKind| Role {
            id: id.to_string(),
            key: Some(id.to_string()),
            name: id.to_string(),
            description: String::new(),
            scope,
            owner_org: None,
            builtin: true,
            permissions: vec![FULL_CONTROL.to_string()],
        };
        let agent = |id: &str, scope: ScopeKind| Role {
            id: id.to_string(),
            key: Some(id.to_string()),
            name: id.to_string(),
            description: String::new(),
            scope,
            owner_org: None,
            builtin: true,
            permissions: [
                "readPipeline",
                "executeJob",
                "writeJobStatus",
                "appendJobLog",
            ]
            .iter()
            .map(ToString::to_string)
            .collect(),
        };
        let runner = Role {
            id: ORGANIZATION_TRIGGER_RUNNER_ROLE.to_string(),
            key: Some(ORGANIZATION_TRIGGER_RUNNER_ROLE.to_string()),
            name: ORGANIZATION_TRIGGER_RUNNER_ROLE.to_string(),
            description: String::new(),
            scope: ScopeKind::Organization,
            owner_org: None,
            builtin: true,
            permissions: vec!["runPipeline".to_string()],
        };
        let developer = Role {
            id: PROJECT_DEVELOPER_ROLE.to_string(),
            key: Some(PROJECT_DEVELOPER_ROLE.to_string()),
            name: PROJECT_DEVELOPER_ROLE.to_string(),
            description: String::new(),
            scope: ScopeKind::Project,
            owner_org: None,
            builtin: true,
            permissions: ["readPipeline", "runPipeline", "createPipeline"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        };
        let org_viewer = Role {
            id: ORGANIZATION_VIEWER_ROLE.to_string(),
            key: Some(ORGANIZATION_VIEWER_ROLE.to_string()),
            name: ORGANIZATION_VIEWER_ROLE.to_string(),
            description: String::new(),
            scope: ScopeKind::Organization,
            owner_org: None,
            builtin: true,
            permissions: ["readOrganization", "listOrganizationMembers"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        };
        vec![
            admin(SYSTEM_ADMIN_ROLE, ScopeKind::System),
            admin(ORGANIZATION_ADMIN_ROLE, ScopeKind::Organization),
            admin(PROJECT_ADMIN_ROLE, ScopeKind::Project),
            agent(ORGANIZATION_AGENT_ROLE, ScopeKind::Organization),
            agent(PROJECT_AGENT_ROLE, ScopeKind::Project),
            runner,
            developer,
            org_viewer,
        ]
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
        async fn create(&self, _role: &Role) -> DomainResult<()> {
            Ok(())
        }
        async fn update(&self, _role: &Role) -> DomainResult<()> {
            Ok(())
        }
        async fn delete(&self, _id: &str) -> DomainResult<()> {
            Ok(())
        }
    }

    struct StubProvider {
        ancestors: ResourceAncestors,
        app_active: bool,
    }

    #[async_trait]
    impl AuthzEntityProvider for StubProvider {
        async fn resource_ancestors(
            &self,
            _resource: &ResourceRef,
        ) -> DomainResult<ResourceAncestors> {
            Ok(self.ancestors.clone())
        }
        async fn app_is_active(&self, _app: &AppId) -> DomainResult<bool> {
            Ok(self.app_active)
        }
    }

    struct StubGrants(Vec<Grant>);

    #[async_trait]
    impl GrantRepository for StubGrants {
        async fn list_all(&self) -> DomainResult<Vec<Grant>> {
            Ok(self.0.clone())
        }
        async fn revoke_all(&self, _p: &Principal, _s: &Scope) -> DomainResult<u64> {
            Ok(0)
        }
        async fn create(&self, _grant: &Grant) -> DomainResult<()> {
            Ok(())
        }
        async fn delete(&self, _id: &str) -> DomainResult<()> {
            Ok(())
        }
    }

    async fn service(
        ancestors: ResourceAncestors,
        grants: Vec<Grant>,
    ) -> CedarPermissionService<StubProvider> {
        CedarPermissionService::new(
            Arc::new(StubProvider {
                ancestors,
                app_active: true,
            }),
            Arc::new(StubRoles(builtin_roles())),
            Arc::new(StubGrants(grants)),
            Arc::new(crate::application::audit::NoopAuditLog),
        )
        .await
        .expect("schema + policies must parse, validate, and link")
    }

    /// Like `service`, but the App principal's backing row is disabled
    /// (`app_is_active` → false). Exercises the liveness gate.
    async fn service_app_inactive(
        ancestors: ResourceAncestors,
        grants: Vec<Grant>,
    ) -> CedarPermissionService<StubProvider> {
        CedarPermissionService::new(
            Arc::new(StubProvider {
                ancestors,
                app_active: false,
            }),
            Arc::new(StubRoles(builtin_roles())),
            Arc::new(StubGrants(grants)),
            Arc::new(crate::application::audit::NoopAuditLog),
        )
        .await
        .expect("schema + policies must parse, validate, and link")
    }

    fn role(name: &str) -> RoleName {
        RoleName::new(name).unwrap()
    }

    // Constructing the service runs Schema::from_cedarschema_str + strict
    // Validator::validate. If the embedded schema/policies drift, this fails.
    #[tokio::test]
    async fn schema_and_policies_validate_and_admin_allows_everything() {
        let svc = service(
            ResourceAncestors::default(),
            vec![Grant::new(
                Principal::User(UserId::new("u-admin")),
                role("system-admin"),
                Scope::System,
            )],
        )
        .await;

        let caller = CallerContext::User(UserId::new("u-admin"));
        assert!(
            svc.check(&caller, Permission::DeleteUser(UserId::new("victim")))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn service_principal_bypasses() {
        let svc = service(ResourceAncestors::default(), vec![]).await;
        let caller = CallerContext::Service(ServiceIdentity::recorder());
        assert!(svc.check(&caller, Permission::CreateJob).await.is_ok());
    }

    #[tokio::test]
    async fn anonymous_denied() {
        let svc = service(ResourceAncestors::default(), vec![]).await;
        assert!(
            svc.check(&CallerContext::Anonymous, Permission::ListUsers)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn project_membership_alone_confers_nothing() {
        // No implicit project tier: belonging to a project grants no capability,
        // every right comes from a grant. The same call succeeds in the test
        // below once a role is actually held.
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u1"));
        assert!(
            svc.check(&caller, Permission::RunPipeline(PipelineId::new("pl1")))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn a_project_role_is_what_confers_rights() {
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![Grant::new(
                Principal::User(UserId::new("u1")),
                role(PROJECT_DEVELOPER_ROLE),
                Scope::Project(ProjectId::new("p1")),
            )],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u1"));
        assert!(
            svc.check(&caller, Permission::RunPipeline(PipelineId::new("pl1")))
                .await
                .is_ok(),
            "the developer role confers runPipeline"
        );
        assert!(
            svc.check(&caller, Permission::DeleteProject(ProjectId::new("p1")))
                .await
                .is_err(),
            "and stops there: deleting the project is an admin action"
        );
    }

    #[tokio::test]
    async fn project_membership_confers_nothing_without_org_membership() {
        // The user was removed from the org but their project-membership row
        // lingers. Membership gates authority: the stale row alone must not
        // authorize anything.
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u1"));
        assert!(
            svc.check(&caller, Permission::RunPipeline(PipelineId::new("pl1")))
                .await
                .is_err(),
            "a project membership without the org membership must confer nothing"
        );
    }

    #[tokio::test]
    async fn project_member_denied_outside_scope() {
        let svc = service(
            // pipeline lives under project p2, which the user is not a member of
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p2")),
                pipeline: None,
            },
            vec![],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u1"));
        assert!(
            svc.check(&caller, Permission::RunPipeline(PipelineId::new("pl1")))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn linked_project_admin_grant_allows_in_scope() {
        let grant = Grant::new(
            Principal::User(UserId::new("u1")),
            role("project-admin"),
            Scope::Project(ProjectId::new("p1")),
        );
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![grant],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u1"));
        assert!(
            svc.check(&caller, Permission::DeletePipeline(PipelineId::new("pl1")))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn a_grant_authorizes_on_its_own() {
        // Grants used to be conditional: a Cedar guard re-read the membership
        // tables on every check, so a grant went inert the moment someone left.
        // With membership gone the grant IS the authority, unconditionally, and
        // removing access means deleting the row. The completeness of that
        // deletion is covered by `revoke_all_access_strips_the_whole_org_subtree`
        // in the organizations persistence tests — that test is what replaces
        // the guard this one used to describe.
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![Grant::new(
                Principal::User(UserId::new("u1")),
                role(PROJECT_ADMIN_ROLE),
                Scope::Project(ProjectId::new("p1")),
            )],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u1"));
        assert!(
            svc.check(&caller, Permission::DeletePipeline(PipelineId::new("pl1")))
                .await
                .is_ok(),
            "the grant alone authorizes: nothing else has to be true"
        );

        // And it reaches only its own scope.
        let elsewhere = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p2")),
                pipeline: None,
            },
            vec![Grant::new(
                Principal::User(UserId::new("u1")),
                role(PROJECT_ADMIN_ROLE),
                Scope::Project(ProjectId::new("p1")),
            )],
        )
        .await;
        assert!(
            elsewhere
                .check(&caller, Permission::DeletePipeline(PipelineId::new("pl9")))
                .await
                .is_err(),
            "a grant on p1 confers nothing on p2"
        );
    }

    #[tokio::test]
    async fn system_scoped_grants_need_no_org_membership() {
        // system-admin authority comes from the System scope, above the org
        // tree: it must keep working inside any org without membership there.
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![Grant::new(
                Principal::User(UserId::new("u-admin")),
                role("system-admin"),
                Scope::System,
            )],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u-admin"));
        assert!(
            svc.check(&caller, Permission::DeletePipeline(PipelineId::new("pl1")))
                .await
                .is_ok(),
            "a system admin acts inside any org without being a member"
        );
    }

    #[tokio::test]
    async fn agent_app_grant_allows_execute_job_in_scope_only() {
        // A machine App with an agent grant on an org may execute jobs on a
        // pipeline beneath it, but not management actions outside the agent set.
        let grant = Grant::new(
            Principal::App(AppId::new("agent-1")),
            role(ORGANIZATION_AGENT_ROLE),
            Scope::Organization(OrganizationId::new("o1")),
        );
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![grant],
        )
        .await;
        let caller = CallerContext::App(AppId::new("agent-1"));
        assert!(
            svc.check(&caller, Permission::ExecuteJob(PipelineId::new("pl1")))
                .await
                .is_ok(),
            "agent app may execute jobs within its granted org"
        );
        assert!(
            svc.check(&caller, Permission::DeletePipeline(PipelineId::new("pl1")))
                .await
                .is_err(),
            "agent role must not confer management actions"
        );
    }

    #[tokio::test]
    async fn trigger_runner_app_grant_allows_run_pipeline_in_scope() {
        // The per-org trigger-runner App holds the `organization-trigger-runner`
        // role at org scope, which confers `runPipeline` and nothing else. This is
        // the load-bearing authz path for triggers: it must authorize firing any
        // pipeline beneath the org — which only links/typechecks because the
        // schema widened `runPipeline` to allow App principals. The runner must
        // NOT thereby gain trigger management.
        let grant = Grant::new(
            Principal::App(AppId::new("trigger-runner-1")),
            role(ORGANIZATION_TRIGGER_RUNNER_ROLE),
            Scope::Organization(OrganizationId::new("o1")),
        );
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![grant],
        )
        .await;
        let caller = CallerContext::App(AppId::new("trigger-runner-1"));
        assert!(
            svc.check(&caller, Permission::RunPipeline(PipelineId::new("pl1")))
                .await
                .is_ok(),
            "trigger-runner App must run pipelines within its granted org",
        );
        assert!(
            svc.check(&caller, Permission::ManageTriggers(PipelineId::new("pl1")))
                .await
                .is_err(),
            "runner App holds only runPipeline, never manageTriggers",
        );
    }

    #[tokio::test]
    async fn narrow_role_grant_allows_only_its_actions_in_scope() {
        // A role conferring only `runPipeline`, granted to Alice on Org A. She may
        // run pipelines beneath the org but the grant confers no other action
        // there.
        let grant = Grant::new(
            Principal::User(UserId::new("alice")),
            role(ORGANIZATION_TRIGGER_RUNNER_ROLE),
            Scope::Organization(OrganizationId::new("o1")),
        );
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![grant],
        )
        .await;
        let caller = CallerContext::User(UserId::new("alice"));
        assert!(
            svc.check(&caller, Permission::RunPipeline(PipelineId::new("pl1")))
                .await
                .is_ok(),
            "a direct runPipeline grant must allow running a pipeline beneath the org",
        );
        assert!(
            svc.check(&caller, Permission::DeletePipeline(PipelineId::new("pl1")))
                .await
                .is_err(),
            "a single-permission grant must not confer any other action",
        );
    }

    #[tokio::test]
    async fn disabled_app_denied_even_with_valid_grant() {
        // Same grant + in-scope resource as the "allows execute" test, but the
        // App's backing row is disabled. The per-action liveness gate in `check`
        // must deny regardless of the otherwise-sufficient agent grant. This is
        // the durable guarantee that a disabled worker cannot keep acting over
        // an already-open stream.
        let grant = Grant::new(
            Principal::App(AppId::new("agent-1")),
            role(ORGANIZATION_AGENT_ROLE),
            Scope::Organization(OrganizationId::new("o1")),
        );
        let svc = service_app_inactive(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![grant],
        )
        .await;
        let caller = CallerContext::App(AppId::new("agent-1"));
        assert!(
            svc.check(&caller, Permission::ExecuteJob(PipelineId::new("pl1")))
                .await
                .is_err(),
            "a disabled app must be denied even where its grant would otherwise allow"
        );
    }

    #[tokio::test]
    async fn agent_app_denied_outside_scope() {
        let grant = Grant::new(
            Principal::App(AppId::new("agent-1")),
            role(ORGANIZATION_AGENT_ROLE),
            Scope::Organization(OrganizationId::new("o1")),
        );
        // Pipeline lives under a different org the app has no grant on.
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o2")),
                project: Some(ProjectId::new("p2")),
                pipeline: None,
            },
            vec![grant],
        )
        .await;
        let caller = CallerContext::App(AppId::new("agent-1"));
        assert!(
            svc.check(&caller, Permission::ExecuteJob(PipelineId::new("pl1")))
                .await
                .is_err(),
            "agent app must be denied outside its granted scope"
        );
    }

    #[tokio::test]
    async fn org_admin_manages_agents_in_its_org_only() {
        // An agent is a specialized app, so agent actions target the org (create/
        // list) or the App resource beneath it (read/stats/delete). An org-admin
        // grant covers all of them within its org via the role template, and
        // nothing outside it. Pure permission — no org-member broadening.
        let grant = Grant::new(
            Principal::User(UserId::new("u1")),
            role(ORGANIZATION_ADMIN_ROLE),
            Scope::Organization(OrganizationId::new("o1")),
        );
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: None,
                pipeline: None,
            },
            vec![grant],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u1"));

        for perm in [
            Permission::CreateAgent(OrganizationId::new("o1")),
            Permission::ListAgents(OrganizationId::new("o1")),
            Permission::ReadApp(AppId::new("agent-1")),
            Permission::ReadAppStats(AppId::new("agent-1")),
            Permission::DeleteApp(AppId::new("agent-1")),
        ] {
            assert!(
                svc.check(&caller, perm.clone()).await.is_ok(),
                "org admin may manage agents in its org: {perm:?}"
            );
        }

        assert!(
            svc.check(&caller, Permission::CreateAgent(OrganizationId::new("o2")))
                .await
                .is_err(),
            "org admin cannot create agents in another org"
        );
    }

    #[tokio::test]
    async fn user_without_grant_denied_agent_actions() {
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: None,
                pipeline: None,
            },
            vec![],
        )
        .await;
        let caller = CallerContext::User(UserId::new("nobody"));
        assert!(
            svc.check(&caller, Permission::ListAgents(OrganizationId::new("o1")))
                .await
                .is_err(),
            "a user with no grant cannot list agents"
        );
        assert!(
            svc.check(&caller, Permission::ReadAppStats(AppId::new("agent-1")))
                .await
                .is_err(),
            "a user with no grant cannot read agent stats"
        );
    }

    #[tokio::test]
    async fn org_admin_manages_grants_in_its_org_only() {
        let grant = Grant::new(
            Principal::User(UserId::new("u1")),
            role(ORGANIZATION_ADMIN_ROLE),
            Scope::Organization(OrganizationId::new("o1")),
        );
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: None,
                pipeline: None,
            },
            vec![grant],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u1"));
        assert!(
            svc.check(
                &caller,
                Permission::ManageOrgGrants(OrganizationId::new("o1"))
            )
            .await
            .is_ok(),
            "org admin may manage grants in its own org"
        );
        assert!(
            svc.check(
                &caller,
                Permission::ManageOrgGrants(OrganizationId::new("o2"))
            )
            .await
            .is_err(),
            "org admin may not manage grants in another org (anti-escalation)"
        );
    }

    #[tokio::test]
    async fn listing_members_does_not_confer_invitation_management() {
        // The leak this guards: whoever can enumerate an organization's people
        // must not thereby be able to enumerate its pending invites, which carry
        // email addresses. `manageInvitations` belongs to the admin role only.
        let viewer = service(
            ResourceAncestors::default(),
            vec![Grant::new(
                Principal::User(UserId::new("viewer")),
                role(ORGANIZATION_VIEWER_ROLE),
                Scope::Organization(OrganizationId::new("o1")),
            )],
        )
        .await;
        let viewer_caller = CallerContext::User(UserId::new("viewer"));
        assert!(
            viewer
                .check(
                    &viewer_caller,
                    Permission::ListOrganizationMembers(OrganizationId::new("o1"))
                )
                .await
                .is_ok(),
            "an organization viewer can list its people"
        );
        assert!(
            viewer
                .check(
                    &viewer_caller,
                    Permission::ManageInvitations(OrganizationId::new("o1"))
                )
                .await
                .is_err(),
            "but must NOT manage its invitations"
        );

        // An org admin may.
        let admin = service(
            ResourceAncestors::default(),
            vec![Grant::new(
                Principal::User(UserId::new("admin")),
                role(ORGANIZATION_ADMIN_ROLE),
                Scope::Organization(OrganizationId::new("o1")),
            )],
        )
        .await;
        assert!(
            admin
                .check(
                    &CallerContext::User(UserId::new("admin")),
                    Permission::ManageInvitations(OrganizationId::new("o1"))
                )
                .await
                .is_ok(),
            "an org admin manages its invitations"
        );
    }

    #[tokio::test]
    async fn org_admin_manages_project_grants_under_its_org() {
        let grant = Grant::new(
            Principal::User(UserId::new("u1")),
            role(ORGANIZATION_ADMIN_ROLE),
            Scope::Organization(OrganizationId::new("o1")),
        );
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![grant],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u1"));
        assert!(
            svc.check(
                &caller,
                Permission::ManageProjectGrants(ProjectId::new("p1"))
            )
            .await
            .is_ok(),
            "org admin may manage grants on a project beneath its org"
        );
    }

    #[tokio::test]
    async fn project_admin_cannot_escalate_to_org_grants() {
        let grant = Grant::new(
            Principal::User(UserId::new("u1")),
            role(PROJECT_ADMIN_ROLE),
            Scope::Project(ProjectId::new("p1")),
        );
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![grant],
        )
        .await;
        let caller = CallerContext::User(UserId::new("u1"));
        assert!(
            svc.check(
                &caller,
                Permission::ManageProjectGrants(ProjectId::new("p1"))
            )
            .await
            .is_ok(),
            "project admin manages grants on its project"
        );
        assert!(
            svc.check(
                &caller,
                Permission::ManageOrgGrants(OrganizationId::new("o1"))
            )
            .await
            .is_err(),
            "project admin cannot escalate to org-level grant management"
        );
    }

    #[tokio::test]
    async fn user_lists_its_own_organizations() {
        // A plain user (no role, no membership) may list its own orgs/projects
        // via the `self` ABAC policy, but not another user's.
        let svc = service(ResourceAncestors::default(), vec![]).await;
        let caller = CallerContext::User(UserId::new("u1"));
        assert!(
            svc.check(
                &caller,
                Permission::ListUserOrganizations(UserId::new("u1"))
            )
            .await
            .is_ok(),
            "a user may list its own organizations"
        );
        assert!(
            svc.check(&caller, Permission::ListUserProjects(UserId::new("u1")))
                .await
                .is_ok(),
            "a user may list its own projects"
        );
        assert!(
            svc.check(&caller, Permission::UpdateUser(UserId::new("u1")))
                .await
                .is_ok(),
            "a user may update its own profile"
        );
        assert!(
            svc.check(&caller, Permission::DeleteUser(UserId::new("u1")))
                .await
                .is_err(),
            "self-deletion is not granted by the self policy"
        );
        assert!(
            svc.check(
                &caller,
                Permission::ListUserOrganizations(UserId::new("u2"))
            )
            .await
            .is_err(),
            "a user may not list another user's organizations"
        );
        assert!(
            svc.check(&caller, Permission::UpdateUser(UserId::new("u2")))
                .await
                .is_err(),
            "a user may not update another user's profile"
        );
    }

    #[tokio::test]
    async fn non_member_without_role_denied() {
        let svc = service(
            ResourceAncestors {
                organization: Some(OrganizationId::new("o1")),
                project: Some(ProjectId::new("p1")),
                pipeline: None,
            },
            vec![],
        )
        .await;
        let caller = CallerContext::User(UserId::new("nobody"));
        assert!(
            svc.check(&caller, Permission::ReadPipeline(PipelineId::new("pl1")))
                .await
                .is_err()
        );
    }
}
