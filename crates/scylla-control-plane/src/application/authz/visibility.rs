use crate::application::authz::grant::{Grant, Scope};
use crate::application::authz::role::FULL_CONTROL;
use crate::application::caller::CallerContext;
use crate::domain::entities::{OrganizationId, ProjectId};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use std::collections::HashMap;

/// Which things a caller may see, expressed as scopes rather than as a list of
/// ids: a grant on an organization covers every project inside it, including
/// ones created after the grant, so enumerating projects here would go stale.
///
/// This is the listing counterpart of a per-item permission check. Both answer
/// the same question; this one answers it for a whole page at once, so the
/// filter can live in SQL and pagination stays honest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    /// Holds the permission at System scope, so nothing is hidden.
    All,
    /// Holds it on these organizations (covering their projects) and on these
    /// individual projects. Empty on both sides means nothing is visible.
    Scoped {
        orgs: Vec<OrganizationId>,
        projects: Vec<ProjectId>,
    },
}

impl Visibility {
    /// Nothing at all — the caller holds the permission nowhere.
    #[must_use]
    pub fn none() -> Self {
        Self::Scoped {
            orgs: Vec::new(),
            projects: Vec::new(),
        }
    }

    /// Whether this can match anything. A listing whose visibility is empty can
    /// skip the query entirely and answer with an empty page.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Scoped { orgs, projects } if orgs.is_empty() && projects.is_empty())
    }
}

/// Resolves what a caller may see, for filtering listings. Kept separate from
/// [`crate::application::authz::service::PermissionService`] because the two
/// answer different shapes of question: one decides a single access, this one
/// describes a set.
#[async_trait]
pub trait VisibilityResolver: Send + Sync {
    /// The scopes at which `caller` holds `permission_key`, directly or through
    /// a role conferring full control.
    async fn visible_scopes(
        &self,
        caller: &CallerContext,
        permission_key: &str,
    ) -> DomainResult<Visibility>;
}

/// Fold a principal's grants into a [`Visibility`] for one permission. Pure, so
/// the rule is testable without a database or a policy engine.
///
/// `role_permissions` maps a role id to its permission keys; a role holding the
/// [`FULL_CONTROL`] sentinel confers every permission within its scope.
#[must_use]
pub fn visibility_from_grants<S: std::hash::BuildHasher>(
    role_permissions: &HashMap<String, Vec<String>, S>,
    grants: &[Grant],
    principal: &crate::application::authz::grant::Principal,
    permission_key: &str,
) -> Visibility {
    let mut orgs = Vec::new();
    let mut projects = Vec::new();

    for grant in grants.iter().filter(|g| &g.principal == principal) {
        let confers = role_permissions
            .get(grant.role.as_str())
            .is_some_and(|perms| {
                perms
                    .iter()
                    .any(|p| p == FULL_CONTROL || p == permission_key)
            });
        if !confers {
            continue;
        }
        match &grant.scope {
            // System covers every organization, so nothing narrower matters.
            Scope::System => return Visibility::All,
            Scope::Organization(id) => orgs.push(id.clone()),
            Scope::Project(id) => projects.push(id.clone()),
        }
    }

    Visibility::Scoped { orgs, projects }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::authz::grant::Principal;
    use crate::domain::entities::UserId;
    use crate::domain::value_objects::role::RoleName;

    fn roles() -> HashMap<String, Vec<String>> {
        HashMap::from([
            ("admin".to_string(), vec![FULL_CONTROL.to_string()]),
            ("viewer".to_string(), vec!["readProject".to_string()]),
            ("runner".to_string(), vec!["runPipeline".to_string()]),
        ])
    }

    fn grant(role: &str, scope: Scope) -> Grant {
        Grant::new(
            Principal::User(UserId::new("alice")),
            RoleName::new(role).unwrap(),
            scope,
        )
    }

    #[test]
    fn a_system_grant_sees_everything() {
        let v = visibility_from_grants(
            &roles(),
            &[grant("admin", Scope::System)],
            &Principal::User(UserId::new("alice")),
            "readProject",
        );
        assert_eq!(v, Visibility::All);
    }

    #[test]
    fn org_and_project_grants_accumulate_by_scope() {
        let grants = vec![
            grant("viewer", Scope::Organization(OrganizationId::new("o1"))),
            grant("viewer", Scope::Project(ProjectId::new("p9"))),
        ];
        let v = visibility_from_grants(
            &roles(),
            &grants,
            &Principal::User(UserId::new("alice")),
            "readProject",
        );
        assert_eq!(
            v,
            Visibility::Scoped {
                orgs: vec![OrganizationId::new("o1")],
                projects: vec![ProjectId::new("p9")],
            }
        );
    }

    #[test]
    fn a_grant_that_does_not_confer_the_permission_is_ignored() {
        // The runner role can launch pipelines but not read a project, so it
        // must not make the project appear in a project listing.
        let v = visibility_from_grants(
            &roles(),
            &[grant("runner", Scope::Project(ProjectId::new("p1")))],
            &Principal::User(UserId::new("alice")),
            "readProject",
        );
        assert!(v.is_empty());
    }

    #[test]
    fn another_principals_grants_are_not_mine() {
        let v = visibility_from_grants(
            &roles(),
            &[grant("admin", Scope::System)],
            &Principal::User(UserId::new("bob")),
            "readProject",
        );
        assert!(v.is_empty());
    }
}
