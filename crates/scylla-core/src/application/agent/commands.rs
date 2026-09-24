//! The agent's writes. One block per command, in the order it runs: the struct, its permission,
//! its payload types, what `Prepare` builds, what `Persist` writes. `DeleteAgent` stages the id
//! alone: the row is not read before the write, as before.

use super::AgentUseCases;
use crate::application::app::mint_app_secret;
use crate::application::{AgentRepository, AppRepository, HashService};
use crate::domain::agent::Agent;
use crate::domain::app::{App, AppCredential, AppName, AppSecret, AppSecretLabel};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, OrganizationId};
use crate::domain::permission::Permission;
use crate::domain::role::RoleName;
use async_trait::async_trait;
use scylla_auth::authz::{Grant, ORGANIZATION_AGENT_ROLE, PolicyControl, Principal, Scope};
use scylla_extension::{
    Authorized, Command, Committed, Deleted, Describe, Draft, Persist, Prepare, Prepared, Run,
};

const DEFAULT_SECRET_LABEL: &str = "default";

#[derive(Debug)]
pub struct CreateAgent {
    pub organization_id: OrganizationId,
    pub name: AppName,
}

/// No `Debug`: `secret` is the plaintext the caller sees once.
pub struct NewAgent {
    pub app: App,
    pub credential: AppCredential,
    pub agent: Agent,
    pub grant: Grant,
    pub secret: AppSecret,
}

pub struct CreatedAgent {
    pub app: App,
    pub secret: AppSecret,
}

impl Describe for CreateAgent {
    fn permission(&self) -> Permission {
        Permission::CreateAgent(self.organization_id.clone())
    }
}

impl Command for CreateAgent {
    type Staged = Draft<NewAgent>;
    type Committed = CreatedAgent;
}

#[async_trait]
impl<A, W, H, PC> Run<Prepare<CreateAgent>> for AgentUseCases<A, W, H, PC>
where
    A: AppRepository,
    W: AgentRepository,
    H: HashService + Send + Sync,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<CreateAgent>) -> DomainResult<Prepared<CreateAgent>> {
        let cmd = input.command();
        let secret = mint_app_secret();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let app = App::create(cmd.organization_id.clone(), cmd.name.clone());
        let credential = AppCredential::create(
            app.id().clone(),
            AppSecretLabel::new(DEFAULT_SECRET_LABEL)?,
            secret_hash,
        );
        let agent = Agent::create(app.id().clone());
        let grant = Grant::new(
            Principal::App(app.id().clone()),
            RoleName::new(ORGANIZATION_AGENT_ROLE)?,
            Scope::Organization(cmd.organization_id.clone()),
        );
        Ok(input.prepared(Draft::new(NewAgent {
            app,
            credential,
            agent,
            grant,
            secret,
        })))
    }
}

#[async_trait]
impl<A, W, H, PC> Run<Persist<CreateAgent>> for AgentUseCases<A, W, H, PC>
where
    A: AppRepository,
    W: AgentRepository,
    H: HashService + Send + Sync,
    PC: PolicyControl,
{
    async fn run(&self, input: Prepared<CreateAgent>) -> DomainResult<Committed<CreateAgent>> {
        input
            .commit(async |draft| {
                let NewAgent {
                    app,
                    credential,
                    agent,
                    grant,
                    secret,
                } = draft.into_inner();
                self.app_repo
                    .provision_agent(&app, &credential, &agent, &grant)
                    .await?;
                self.policy_control.reload().await?;
                Ok(CreatedAgent { app, secret })
            })
            .await
    }
}

#[derive(Debug)]
pub struct DeleteAgent {
    pub id: AppId,
}

impl Describe for DeleteAgent {
    fn permission(&self) -> Permission {
        Permission::DeleteApp(self.id.clone())
    }
}

impl Command for DeleteAgent {
    type Staged = AppId;
    type Committed = Deleted<AppId>;
}

#[async_trait]
impl<A, W, H, PC> Run<Prepare<DeleteAgent>> for AgentUseCases<A, W, H, PC>
where
    A: AppRepository,
    W: AgentRepository,
    H: HashService + Send + Sync,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<DeleteAgent>) -> DomainResult<Prepared<DeleteAgent>> {
        let id = input.command().id.clone();
        Ok(input.prepared(id))
    }
}

#[async_trait]
impl<A, W, H, PC> Run<Persist<DeleteAgent>> for AgentUseCases<A, W, H, PC>
where
    A: AppRepository,
    W: AgentRepository,
    H: HashService + Send + Sync,
    PC: PolicyControl,
{
    async fn run(&self, input: Prepared<DeleteAgent>) -> DomainResult<Committed<DeleteAgent>> {
        input
            .commit(async |id| {
                // Drop the stream first so a removed agent stops at once; the delete cascades the rest.
                self.registry.disconnect(&id);
                self.app_repo.delete(&id).await?;
                self.policy_control.reload().await?;
                Ok(Deleted::new(id))
            })
            .await
    }
}
