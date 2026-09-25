//! The role's writes. One block per command, in the order it runs: the struct, its access,
//! its payload types, what `Prepare` checks and builds, what `Persist` writes. The policy reload
//! sits next to the write that changes a role Cedar emits.

use super::RoleUseCases;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::permission::Permission;
use async_trait::async_trait;
use scylla_auth::authz::{Role, ScopeKind, validate_role_permissions};
use scylla_extension::{
    Access, Authorized, Command, Committed, Deleted, Describe, Draft, Persist, Prepare, Prepared,
    Run,
};

#[derive(Debug)]
pub struct CreateRole {
    pub name: String,
    pub description: String,
    pub scope: ScopeKind,
    pub permissions: Vec<String>,
}

impl Describe for CreateRole {
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageRoles)
    }
}

impl Command for CreateRole {
    type Staged = Draft<Role>;
    type Committed = Role;
}

#[async_trait]
impl Run<Prepare<CreateRole>> for RoleUseCases {
    async fn run(&self, input: Authorized<CreateRole>) -> DomainResult<Prepared<CreateRole>> {
        let cmd = input.command();
        validate_role_permissions(&cmd.permissions, cmd.scope)?;
        let role = Role::new_custom(
            cmd.name.clone(),
            cmd.description.clone(),
            cmd.scope,
            cmd.permissions.clone(),
        );
        Ok(input.prepared(Draft::new(role)))
    }
}

#[async_trait]
impl Run<Persist<CreateRole>> for RoleUseCases {
    async fn run(&self, input: Prepared<CreateRole>) -> DomainResult<Committed<CreateRole>> {
        input
            .commit(async |draft| {
                let role = draft.into_inner();
                self.role_repo.create(&role).await?;
                self.policy_control.reload().await?;
                Ok(role)
            })
            .await
    }
}

#[derive(Debug)]
pub struct UpdateRole {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
}

impl Describe for UpdateRole {
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageRoles)
    }
}

impl Command for UpdateRole {
    type Staged = Draft<Role>;
    type Committed = Role;
}

#[async_trait]
impl Run<Prepare<UpdateRole>> for RoleUseCases {
    async fn run(&self, input: Authorized<UpdateRole>) -> DomainResult<Prepared<UpdateRole>> {
        let cmd = input.command();
        let mut role = self
            .role_repo
            .get(&cmd.id)
            .await?
            .ok_or_else(|| DomainError::not_found("role", &cmd.id))?;
        validate_role_permissions(&cmd.permissions, role.scope)?;
        role.name.clone_from(&cmd.name);
        role.description.clone_from(&cmd.description);
        role.permissions.clone_from(&cmd.permissions);
        Ok(input.prepared(Draft::new(role)))
    }
}

#[async_trait]
impl Run<Persist<UpdateRole>> for RoleUseCases {
    async fn run(&self, input: Prepared<UpdateRole>) -> DomainResult<Committed<UpdateRole>> {
        input
            .commit(async |draft| {
                let role = draft.into_inner();
                self.role_repo.update(&role).await?;
                self.policy_control.reload().await?;
                Ok(role)
            })
            .await
    }
}

#[derive(Debug)]
pub struct DeleteRole {
    pub id: String,
}

impl Describe for DeleteRole {
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageRoles)
    }
}

impl Command for DeleteRole {
    type Staged = Role;
    type Committed = Deleted<Role>;
}

#[async_trait]
impl Run<Prepare<DeleteRole>> for RoleUseCases {
    async fn run(&self, input: Authorized<DeleteRole>) -> DomainResult<Prepared<DeleteRole>> {
        let id = &input.command().id;
        let role = self
            .role_repo
            .get(id)
            .await?
            .ok_or_else(|| DomainError::not_found("role", id))?;
        if role.builtin {
            return Err(DomainError::business_rule(
                "builtin roles cannot be deleted",
            ));
        }
        // A role still granted must be unassigned first; those grants would silently stop working.
        let grants = self.grant_repo.list_all().await?;
        if grants.iter().any(|g| g.role.as_str() == id) {
            return Err(DomainError::business_rule(
                "role is still granted to one or more principals; revoke those grants first",
            ));
        }
        Ok(input.prepared(role))
    }
}

#[async_trait]
impl Run<Persist<DeleteRole>> for RoleUseCases {
    async fn run(&self, input: Prepared<DeleteRole>) -> DomainResult<Committed<DeleteRole>> {
        input
            .commit(async |role| {
                self.role_repo.delete(&role.id).await?;
                self.policy_control.reload().await?;
                Ok(Deleted::new(role))
            })
            .await
    }
}
