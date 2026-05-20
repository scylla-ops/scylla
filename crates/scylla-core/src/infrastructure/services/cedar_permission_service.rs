use crate::application::PermissionService;
use crate::application::audit::{AuditDecision, AuditEntry, AuditLog};
use crate::application::caller::CallerContext;
use crate::application::permission::entity_provider::{AuthzEntityProvider, PrincipalAuthz};
use crate::application::permission::grant::{Grant, GrantRepository, GrantScope};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::{Permission, ResourceRef};
use async_trait::async_trait;
use chrono::Utc;
use cedar_policy::{
    Authorizer, Context, Decision, Entities, Entity, EntityUid, PolicyId, PolicySet, Request,
    RestrictedExpression, Schema, SlotId, Template, ValidationMode, Validator,
};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{info, instrument, warn};

const SCHEMA_SRC: &str = include_str!("cedar/schema.cedarschema");
const POLICIES_SRC: &str = include_str!("cedar/policies.cedar");
/// Shared body for every scoped role; registered once per role id below.
const ROLE_TEMPLATE_SRC: &str = include_str!("cedar/role_template.cedar");
/// Role names that map to a linkable Cedar template. A grant whose `role` is
/// not listed here is skipped at link time (logged).
const TEMPLATE_ROLES: &[&str] = &["organization-admin", "project-admin"];

/// Cedar-backed authorization. Static policies + schema are compiled into the
/// binary and validated at construction; explicit scoped grants are linked from
/// the grant store as Cedar template instances. The principal's roles and
/// org/project memberships, plus the resource's ancestor chain, are materialised
/// per request via the [`AuthzEntityProvider`].
pub struct CedarPermissionService<EP: AuthzEntityProvider> {
    policies: PolicySet,
    authorizer: Authorizer,
    entity_provider: Arc<EP>,
    audit: Arc<dyn AuditLog>,
}

impl<EP: AuthzEntityProvider> CedarPermissionService<EP> {
    /// Parse + validate policies against the schema, then link one template
    /// instance per stored grant. Fails fast if the embedded policies don't
    /// typecheck — catching policy/schema drift at startup (and in tests).
    pub async fn new<G: GrantRepository>(
        entity_provider: Arc<EP>,
        grant_repo: &G,
        audit: Arc<dyn AuditLog>,
    ) -> DomainResult<Self> {
        let (schema, _warnings) = Schema::from_cedarschema_str(SCHEMA_SRC)
            .map_err(|e| DomainError::Internal(format!("cedar schema parse: {e}")))?;

        let mut policies = PolicySet::from_str(POLICIES_SRC)
            .map_err(|e| DomainError::Internal(format!("cedar policy parse: {e}")))?;

        // Register the scoped-role template under each grantable role id so
        // grants can link instances by role name.
        for role in TEMPLATE_ROLES {
            let template = Template::parse(Some(PolicyId::new(*role)), ROLE_TEMPLATE_SRC)
                .map_err(|e| DomainError::Internal(format!("cedar template parse {role}: {e}")))?;
            policies
                .add_template(template)
                .map_err(|e| DomainError::Internal(format!("cedar add template {role}: {e}")))?;
        }

        let result = Validator::new(schema).validate(&policies, ValidationMode::Strict);
        if !result.validation_passed() {
            let errs: Vec<String> = result.validation_errors().map(ToString::to_string).collect();
            return Err(DomainError::Internal(format!(
                "cedar policy validation failed: {errs:?}"
            )));
        }

        let grants = grant_repo.list_all().await?;
        for grant in &grants {
            if let Err(e) = Self::link_grant(&mut policies, grant) {
                warn!(grant_id = %grant.id, error = %e, "skipping unlinkable grant");
            }
        }

        Ok(Self {
            policies,
            authorizer: Authorizer::new(),
            entity_provider,
            audit,
        })
    }

    /// Link a stored grant as a template instance. Template id == role name
    /// (`project-admin` / `organization-admin`); `?principal` → the user,
    /// `?resource` → the granted scope.
    fn link_grant(policies: &mut PolicySet, grant: &Grant) -> DomainResult<()> {
        let principal_uid = euid("Scylla::User", grant.user_id.as_str())?;
        let resource_uid = match &grant.scope {
            GrantScope::Organization(id) => euid("Scylla::Organization", id.as_str())?,
            GrantScope::Project(id) => euid("Scylla::Project", id.as_str())?,
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
                let uid = euid("Scylla::User", id.as_str())?;
                let authz = self.entity_provider.principal_authz(id).await?;
                let mut entities = role_entities(&authz)?;
                entities.push(user_entity(uid.clone(), &authz)?);
                Ok((uid, entities))
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
        if let Some(o) = &org_uid {
            entities.push(Entity::new_no_attrs(o.clone(), HashSet::new()));
        }
        if let Some(p) = &project_uid {
            entities.push(Entity::new_no_attrs(p.clone(), parent_set(org_uid.as_ref())));
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
            ResourceRef::Project(_) => org_uid.as_ref(),
            _ => None,
        };

        // The leaf entity itself. A `User` resource needs the schema's required
        // attrs (empty here — policies never read resource memberships).
        let leaf = match resource {
            ResourceRef::User(_) => user_entity(uid.clone(), &PrincipalAuthz::default())?,
            _ => Entity::new_no_attrs(uid.clone(), parent_set(leaf_parent)),
        };
        entities.push(leaf);

        Ok((uid, entities))
    }
}

#[async_trait]
impl<EP: AuthzEntityProvider + 'static> PermissionService for CedarPermissionService<EP> {
    #[instrument(skip(self, caller, perm), fields(caller = ?caller, action = perm.action()))]
    async fn check(&self, caller: &CallerContext, perm: Permission) -> DomainResult<bool> {
        let (principal_uid, principal_entities) = self.principal_entities(caller).await?;
        let resource = perm.resource();
        let (resource_uid, resource_entities) = self.resource_entities(&resource).await?;
        let action_uid = euid("Scylla::Action", perm.action())?;

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

        let response = self
            .authorizer
            .is_authorized(&request, &self.policies, &entities);

        // The Cedar policy ids that determined the verdict (admin/ABAC rule or a
        // linked grant) — captured for the audit trail.
        let policies: Vec<String> = response
            .diagnostics()
            .reason()
            .map(ToString::to_string)
            .collect();
        let (principal_kind, principal_id) = principal_parts(caller);
        let (resource_kind, resource_id) = resource_parts(&resource);

        let (audit_decision, reason, result) = match response.decision() {
            Decision::Allow => (AuditDecision::Allow, None, Ok(true)),
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
                    Err(DomainError::Forbidden("Permission denied".to_string())),
                )
            }
        };

        // Live console trail (the `audit` tracing target) …
        match audit_decision {
            AuditDecision::Allow => info!(
                target: "audit",
                who = %caller, action = perm.action(), resource = %resource,
                decision = "allow", policies = ?policies, "action authorized"
            ),
            AuditDecision::Deny => warn!(
                target: "audit",
                who = %caller, action = perm.action(), resource = %resource,
                decision = "deny", policies = ?policies, "action denied"
            ),
        }

        // … and the persistent trail (out-of-band insert).
        self.audit.record(AuditEntry {
            occurred_at: Utc::now(),
            principal_kind,
            principal_id,
            action: perm.action(),
            resource_kind,
            resource_id,
            decision: audit_decision,
            policies,
            reason,
        });

        result
    }
}

// ── helpers ──────────────────────────────────────────────────────────────

fn euid(type_name: &str, id: &str) -> DomainResult<EntityUid> {
    EntityUid::from_str(&format!("{type_name}::\"{id}\""))
        .map_err(|e| DomainError::Internal(format!("cedar uid {type_name}::{id}: {e}")))
}

fn parent_set(parent: Option<&EntityUid>) -> HashSet<EntityUid> {
    parent.cloned().into_iter().collect()
}

fn uid_set<T: AsRef<str>>(type_name: &str, ids: &[T]) -> DomainResult<RestrictedExpression> {
    let mut exprs = Vec::with_capacity(ids.len());
    for id in ids {
        exprs.push(RestrictedExpression::new_entity_uid(euid(
            type_name,
            id.as_ref(),
        )?));
    }
    Ok(RestrictedExpression::new_set(exprs))
}

fn role_entities(authz: &PrincipalAuthz) -> DomainResult<Vec<Entity>> {
    authz
        .roles
        .iter()
        .map(|r| {
            let uid = euid("Scylla::Role", r.as_str())?;
            Ok(Entity::new_no_attrs(uid, HashSet::new()))
        })
        .collect()
}

fn user_entity(uid: EntityUid, authz: &PrincipalAuthz) -> DomainResult<Entity> {
    let mut parents = HashSet::new();
    for role in &authz.roles {
        parents.insert(euid("Scylla::Role", role.as_str())?);
    }
    let attrs = HashMap::from([
        (
            "memberOrgs".to_string(),
            uid_set("Scylla::Organization", &authz.member_orgs)?,
        ),
        (
            "memberProjects".to_string(),
            uid_set("Scylla::Project", &authz.member_projects)?,
        ),
    ]);
    Entity::new(uid, attrs, parents)
        .map_err(|e| DomainError::Internal(format!("cedar user entity: {e}")))
}

fn resource_uid(resource: &ResourceRef) -> DomainResult<EntityUid> {
    match resource {
        ResourceRef::System => euid("Scylla::System", "root"),
        ResourceRef::User(id) => euid("Scylla::User", id.as_str()),
        ResourceRef::Organization(id) => euid("Scylla::Organization", id.as_str()),
        ResourceRef::Project(id) => euid("Scylla::Project", id.as_str()),
        ResourceRef::Pipeline(id) => euid("Scylla::Pipeline", id.as_str()),
        ResourceRef::Job(id) => euid("Scylla::Job", id.as_str()),
        ResourceRef::Agent(id) => euid("Scylla::Agent", id.as_str()),
    }
}

/// `(kind, id)` decomposition of a caller for the audit trail.
fn principal_parts(caller: &CallerContext) -> (&'static str, Option<String>) {
    match caller {
        CallerContext::User(id) => ("user", Some(id.as_str().to_string())),
        CallerContext::Service(svc) => ("service", Some(svc.as_str().to_string())),
        CallerContext::Anonymous => ("anonymous", None),
    }
}

/// `(kind, id)` decomposition of a resource for the audit trail.
fn resource_parts(resource: &ResourceRef) -> (&'static str, Option<String>) {
    match resource {
        ResourceRef::System => ("system", None),
        ResourceRef::User(id) => ("user", Some(id.as_str().to_string())),
        ResourceRef::Organization(id) => ("organization", Some(id.as_str().to_string())),
        ResourceRef::Project(id) => ("project", Some(id.as_str().to_string())),
        ResourceRef::Pipeline(id) => ("pipeline", Some(id.as_str().to_string())),
        ResourceRef::Job(id) => ("job", Some(id.as_str().to_string())),
        ResourceRef::Agent(id) => ("agent", Some(id.as_str().to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::caller::ServiceIdentity;
    use crate::application::permission::entity_provider::ResourceAncestors;
    use crate::domain::entities::{OrganizationId, PipelineId, ProjectId, UserId};
    use crate::domain::value_objects::role::name::RoleName;

    struct StubProvider {
        authz: PrincipalAuthz,
        ancestors: ResourceAncestors,
    }

    #[async_trait]
    impl AuthzEntityProvider for StubProvider {
        async fn principal_authz(&self, _user: &UserId) -> DomainResult<PrincipalAuthz> {
            Ok(self.authz.clone())
        }
        async fn resource_ancestors(
            &self,
            _resource: &ResourceRef,
        ) -> DomainResult<ResourceAncestors> {
            Ok(self.ancestors.clone())
        }
    }

    struct StubGrants(Vec<Grant>);

    #[async_trait]
    impl GrantRepository for StubGrants {
        async fn list_all(&self) -> DomainResult<Vec<Grant>> {
            Ok(self.0.clone())
        }
        async fn create(&self, _grant: &Grant) -> DomainResult<()> {
            Ok(())
        }
        async fn delete(&self, _id: &str) -> DomainResult<()> {
            Ok(())
        }
    }

    async fn service(
        authz: PrincipalAuthz,
        ancestors: ResourceAncestors,
        grants: Vec<Grant>,
    ) -> CedarPermissionService<StubProvider> {
        CedarPermissionService::new(
            Arc::new(StubProvider { authz, ancestors }),
            &StubGrants(grants),
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
            PrincipalAuthz {
                roles: vec![role("admin")],
                ..Default::default()
            },
            ResourceAncestors::default(),
            vec![],
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
        let svc = service(PrincipalAuthz::default(), ResourceAncestors::default(), vec![]).await;
        let caller = CallerContext::Service(ServiceIdentity::recorder());
        assert!(svc.check(&caller, Permission::CreateJob).await.is_ok());
    }

    #[tokio::test]
    async fn anonymous_denied() {
        let svc = service(PrincipalAuthz::default(), ResourceAncestors::default(), vec![]).await;
        assert!(
            svc.check(&CallerContext::Anonymous, Permission::ListUsers)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn project_member_allowed_within_scope() {
        let svc = service(
            PrincipalAuthz {
                roles: vec![],
                member_orgs: vec![],
                member_projects: vec![ProjectId::new("p1")],
            },
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
                .is_ok()
        );
    }

    #[tokio::test]
    async fn project_member_denied_outside_scope() {
        let svc = service(
            PrincipalAuthz {
                roles: vec![],
                member_orgs: vec![],
                member_projects: vec![ProjectId::new("p1")],
            },
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
            UserId::new("u1"),
            role("project-admin"),
            GrantScope::Project(ProjectId::new("p1")),
        );
        let svc = service(
            PrincipalAuthz::default(),
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
    async fn non_member_without_role_denied() {
        let svc = service(
            PrincipalAuthz::default(),
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

