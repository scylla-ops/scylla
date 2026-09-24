pub mod commands;
pub mod queries;

pub use commands::{CreateRole, DeleteRole, UpdateRole};
pub use queries::{GetEffectivePermissions, GetRole, ListAuthzVocabulary, ListRoles};

use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use derive_more::Constructor;
use scylla_auth::authz::{
    EffectiveScope, FULL_CONTROL, GrantRepository, PolicyControl, Principal, RoleRepository, Scope,
};
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use tracing::instrument;

/// The role aggregate's stage runners, one block per action in `commands.rs` and `queries.rs`.
/// `my_permissions` stays outside the pipeline: a caller reads its own grants and no permission
/// is asked.
#[derive(Constructor)]
pub struct RoleUseCases<RR: RoleRepository, GR: GrantRepository, PC: PolicyControl> {
    pub(super) role_repo: Arc<RR>,
    pub(super) grant_repo: Arc<GR>,
    pub(super) policy_control: Arc<PC>,
}

impl<RR: RoleRepository, GR: GrantRepository, PC: PolicyControl> RoleUseCases<RR, GR, PC> {
    /// Service and Anonymous are refused: an empty list would read as "no permissions".
    #[instrument(skip(self, caller))]
    pub async fn my_permissions(
        &self,
        caller: &CallerContext,
    ) -> DomainResult<Vec<EffectiveScope>> {
        let principal = Principal::from_caller(caller).ok_or_else(|| {
            DomainError::Forbidden("this caller is not a principal that holds grants".to_string())
        })?;
        self.effective_scopes(&principal).await
    }

    /// Grants as bound per scope: a System grant is not re-listed under every org and project.
    pub(super) async fn effective_scopes(
        &self,
        principal: &Principal,
    ) -> DomainResult<Vec<EffectiveScope>> {
        let role_perms: HashMap<String, Vec<String>> = self
            .role_repo
            .list_all()
            .await?
            .into_iter()
            .map(|r| (r.id, r.permissions))
            .collect();
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
