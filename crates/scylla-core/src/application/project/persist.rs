//! The store's half of each command: the write happens inside `commit` and nothing else does.
//! The policy reload sits next to the write that changes the grant set; a `Listener` on
//! `Persist` would be its next home once a failed reload no longer needs to fail the call.

use super::commands::{CreateProject, DeleteProject, NewProject, SetProjectActive, UpdateProject};
use super::use_case::ProjectUseCases;
use crate::application::{ProjectRepository, UserRepository};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use scylla_auth::authz::{PermissionService, PolicyControl};
use scylla_extension::{Committed, Deleted, Persist, Prepared, Run};

#[async_trait]
impl<P, U, PS, PC> Run<Persist<CreateProject>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Prepared<CreateProject>) -> DomainResult<Committed<CreateProject>> {
        input
            .commit(async |draft| {
                let NewProject { project, owner } = draft.into_inner();
                match owner {
                    Some(grant) => {
                        self.project_repo
                            .provision_with_owner(&project, &grant)
                            .await?;
                        self.policy_control.reload().await?;
                        Ok(project)
                    }
                    None => self.project_repo.create(&project).await,
                }
            })
            .await
    }
}

#[async_trait]
impl<P, U, PS, PC> Run<Persist<UpdateProject>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Prepared<UpdateProject>) -> DomainResult<Committed<UpdateProject>> {
        input
            .commit(async |draft| self.project_repo.update(&draft.into_inner()).await)
            .await
    }
}

#[async_trait]
impl<P, U, PS, PC> Run<Persist<SetProjectActive>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(
        &self,
        input: Prepared<SetProjectActive>,
    ) -> DomainResult<Committed<SetProjectActive>> {
        input
            .commit(async |draft| self.project_repo.update(&draft.into_inner()).await)
            .await
    }
}

#[async_trait]
impl<P, U, PS, PC> Run<Persist<DeleteProject>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Prepared<DeleteProject>) -> DomainResult<Committed<DeleteProject>> {
        input
            .commit(async |project| {
                self.project_repo.delete(project.id()).await?;
                self.policy_control.reload().await?;
                Ok(Deleted::new(project))
            })
            .await
    }
}
