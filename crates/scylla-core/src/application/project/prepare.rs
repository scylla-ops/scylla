//! The use case's half of each command: read and build, never write.

use super::commands::{CreateProject, DeleteProject, NewProject, SetProjectActive, UpdateProject};
use super::use_case::ProjectUseCases;
use crate::application::{ProjectRepository, UserRepository};
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::project::Project;
use crate::domain::role::RoleName;
use async_trait::async_trait;
use scylla_auth::authz::{
    Grant, PROJECT_ADMIN_ROLE, PermissionService, PolicyControl, Principal, Scope,
};
use scylla_extension::{Authorized, Draft, Prepare, Prepared, Run};

#[async_trait]
impl<P, U, PS, PC> Run<Prepare<CreateProject>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<CreateProject>) -> DomainResult<Prepared<CreateProject>> {
        let cmd = input.command();
        let project = Project::create(
            cmd.name.clone(),
            cmd.description.clone(),
            cmd.organization_id.clone(),
        )?;
        let owner = match input.caller() {
            CallerContext::User(user_id) => Some(Grant::new(
                Principal::User(user_id.clone()),
                RoleName::new(PROJECT_ADMIN_ROLE)?,
                Scope::Project(project.id().clone()),
            )),
            _ => None,
        };
        Ok(input.prepared(Draft::new(NewProject { project, owner })))
    }
}

#[async_trait]
impl<P, U, PS, PC> Run<Prepare<UpdateProject>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<UpdateProject>) -> DomainResult<Prepared<UpdateProject>> {
        let cmd = input.command();
        let mut project = self.project_repo.find_by_id(&cmd.id).await?;
        if let Some(name) = &cmd.name {
            project.update_name(name.clone())?;
        }
        if let Some(description) = &cmd.description {
            project.update_description(description.clone())?;
        }
        Ok(input.prepared(Draft::new(project)))
    }
}

#[async_trait]
impl<P, U, PS, PC> Run<Prepare<SetProjectActive>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(
        &self,
        input: Authorized<SetProjectActive>,
    ) -> DomainResult<Prepared<SetProjectActive>> {
        let cmd = input.command();
        let mut project = self.project_repo.find_by_id(&cmd.id).await?;
        project.set_active(cmd.is_active);
        Ok(input.prepared(Draft::new(project)))
    }
}

#[async_trait]
impl<P, U, PS, PC> Run<Prepare<DeleteProject>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<DeleteProject>) -> DomainResult<Prepared<DeleteProject>> {
        let project = self.project_repo.find_by_id(&input.command().id).await?;
        Ok(input.prepared(project))
    }
}
