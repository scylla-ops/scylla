pub mod commands;
pub mod queries;

pub use commands::{CreateRole, DeleteRole, UpdateRole};
pub use queries::{
    GetEffectivePermissions, GetMyPermissions, GetRole, ListAuthzVocabulary, ListRoles,
};

use crate::domain::errors::DomainResult;
use derive_more::Constructor;
use scylla_auth::authz::{
    EffectiveScope, FULL_CONTROL, GrantRepository, PolicyControl, Principal, RoleRepository, Scope,
    permissions_by_role,
};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The role aggregate's stage runners, one block per action in `commands.rs` and `queries.rs`.
#[derive(Constructor)]
pub struct RoleUseCases {
    pub(super) role_repo: Arc<dyn RoleRepository>,
    pub(super) grant_repo: Arc<dyn GrantRepository>,
    pub(super) policy_control: Arc<dyn PolicyControl>,
}

impl RoleUseCases {
    /// Grants as bound per scope: a System grant is not re-listed under every org and project.
    pub(super) async fn effective_scopes(
        &self,
        principal: &Principal,
    ) -> DomainResult<Vec<EffectiveScope>> {
        let role_perms = permissions_by_role(self.role_repo.as_ref()).await?;
        let grants = self.grant_repo.list_all().await?;

        let mut groups: Vec<(Scope, BTreeSet<String>)> = Vec::new();
        for grant in grants.iter().filter(|g| g.principal == *principal) {
            let perms: Vec<String> = role_perms
                .get(grant.role.as_str())
                .cloned()
                .unwrap_or_default();
            if let Some(idx) = groups.iter().position(|(s, _)| *s == grant.scope) {
                groups[idx].1.extend(perms);
            } else {
                groups.push((grant.scope.clone(), perms.into_iter().collect()));
            }
        }

        Ok(groups
            .into_iter()
            .map(|(scope, set)| {
                let full_control = set.contains(FULL_CONTROL);
                EffectiveScope {
                    scope,
                    full_control,
                    permissions: if full_control {
                        Vec::new()
                    } else {
                        set.into_iter().collect()
                    },
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests;
