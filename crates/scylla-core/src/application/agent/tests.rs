//! The agent's admin actions through the engine, on stub ports.

use super::*;
use crate::domain::agent::{Agent, AgentHost};
use crate::domain::app::{App, AppCredential, AppName};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, OrganizationId};
use crate::domain::permission::Permission;
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService, actions};
use crate::test_support::stubs::{CountingPolicy, StubHash, StubRegistry, alice};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla_auth::authz::{Grant, ORGANIZATION_AGENT_ROLE, PermissionService, Principal, Scope};
use scylla_extension::Actions;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct StubApps {
    rows: Mutex<HashMap<AppId, App>>,
    credentials: Mutex<Vec<AppCredential>>,
    grants: Mutex<Vec<Grant>>,
    agents: Arc<StubAgents>,
}

#[async_trait]
impl AppRepository for StubApps {
    async fn create_app(&self, _: &App, _: &AppCredential) -> DomainResult<()> {
        unreachable!("an agent is provisioned, not created")
    }
    async fn provision_agent(
        &self,
        app: &App,
        credential: &AppCredential,
        agent: &Agent,
        grant: &Grant,
    ) -> DomainResult<()> {
        self.rows
            .lock()
            .unwrap()
            .insert(app.id().clone(), app.clone());
        self.credentials.lock().unwrap().push(credential.clone());
        self.grants.lock().unwrap().push(grant.clone());
        self.agents.rows.lock().unwrap().push(agent.clone());
        Ok(())
    }
    async fn provision(&self, _: &App, _: &AppCredential, _: &Grant) -> DomainResult<()> {
        unreachable!("an agent is provisioned with its agent row")
    }
    async fn find_by_id(&self, id: &AppId) -> DomainResult<App> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("App", id.to_string()))
    }
    async fn list_by_organization(&self, _: &OrganizationId) -> DomainResult<Vec<App>> {
        unreachable!("agents are listed through the agent repository")
    }
    async fn set_active(&self, _: &AppId, _: bool) -> DomainResult<()> {
        unreachable!("no activation in an agent action")
    }
    async fn delete(&self, id: &AppId) -> DomainResult<()> {
        self.rows.lock().unwrap().remove(id);
        self.agents
            .rows
            .lock()
            .unwrap()
            .retain(|a| a.app_id() != id);
        Ok(())
    }
}

#[derive(Default)]
struct StubAgents {
    rows: Mutex<Vec<Agent>>,
    organizations: Mutex<HashMap<AppId, OrganizationId>>,
}

#[async_trait]
impl AgentRepository for StubAgents {
    async fn find_by_app_id(&self, app_id: &AppId) -> DomainResult<Agent> {
        self.rows
            .lock()
            .unwrap()
            .iter()
            .find(|a| a.app_id() == app_id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("Agent", app_id.to_string()))
    }
    async fn list_by_organization(&self, org_id: &OrganizationId) -> DomainResult<Vec<Agent>> {
        let organizations = self.organizations.lock().unwrap();
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|a| organizations.get(a.app_id()) == Some(org_id))
            .cloned()
            .collect())
    }
    async fn touch_last_seen(&self, _: &AppId, _: DateTime<Utc>) -> DomainResult<()> {
        unreachable!("only the agent stream touches last_seen")
    }
    async fn record_host(&self, _: &AppId, _: &AgentHost) -> DomainResult<()> {
        unreachable!("only the agent stream records its host")
    }
    async fn agent_stats(&self, _: &AppId) -> DomainResult<AgentStats> {
        Ok(AgentStats {
            total: 3,
            ..AgentStats::default()
        })
    }
}

struct Lab {
    actions: Actions,
    uc: AgentUseCases,
    apps: Arc<StubApps>,
    agents: Arc<StubAgents>,
    hash: Arc<StubHash>,
    registry: Arc<StubRegistry>,
    policy: Arc<CountingPolicy>,
}

impl Lab {
    async fn create(&self) -> DomainResult<CreatedAgent> {
        let created = self
            .actions
            .run(
                &self.uc,
                &alice(),
                CreateAgent {
                    organization_id: org(),
                    name: AppName::new("runner-1").unwrap(),
                },
            )
            .await?;
        self.agents
            .organizations
            .lock()
            .unwrap()
            .insert(created.app.id().clone(), org());
        Ok(created)
    }
}

fn lab(permissions: Arc<dyn PermissionService>) -> Lab {
    let agents = Arc::new(StubAgents::default());
    let apps = Arc::new(StubApps {
        agents: agents.clone(),
        ..StubApps::default()
    });
    let hash = Arc::new(StubHash::secrets());
    let registry = Arc::new(StubRegistry::default());
    let policy = Arc::new(CountingPolicy::default());
    Lab {
        actions: actions(permissions),
        uc: AgentUseCases::new(
            apps.clone(),
            agents.clone(),
            hash.clone(),
            policy.clone(),
            registry.clone(),
        ),
        apps,
        agents,
        hash,
        registry,
        policy,
    }
}

fn org() -> OrganizationId {
    OrganizationId::new("acme")
}

#[tokio::test]
async fn a_create_provisions_the_app_its_secret_its_agent_row_and_its_grant_then_reloads() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());

    let created = lab.create().await.unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::CreateAgent(org())]
    );
    assert!(!created.secret.as_str().is_empty());
    let id = created.app.id().clone();
    assert!(lab.apps.rows.lock().unwrap().contains_key(&id));
    let credentials = lab.apps.credentials.lock().unwrap();
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].label().as_str(), "default");
    assert_eq!(lab.agents.rows.lock().unwrap()[0].app_id(), &id);
    let grants = lab.apps.grants.lock().unwrap();
    assert_eq!(grants[0].principal, Principal::App(id));
    assert_eq!(grants[0].role.as_str(), ORGANIZATION_AGENT_ROLE);
    assert_eq!(grants[0].scope, Scope::Organization(org()));
    assert_eq!(lab.policy.reloads(), 1);
}

#[tokio::test]
async fn a_denied_create_never_hashes_or_persists() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));

    let err = lab.create().await.err().unwrap();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert_eq!(lab.hash.hashed(), 0);
    assert!(lab.apps.rows.lock().unwrap().is_empty());
    assert_eq!(lab.policy.reloads(), 0);
}

#[tokio::test]
async fn reads_check_their_permission_and_join_the_live_registry() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let id = lab.create().await.unwrap().app.id().clone();
    lab.registry.connect(&id);

    let views = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            ListAgents {
                organization_id: org(),
            },
        )
        .await
        .unwrap();
    let view = lab
        .actions
        .run(&lab.uc, &alice(), GetAgent { id: id.clone() })
        .await
        .unwrap();
    let stats = lab
        .actions
        .run(&lab.uc, &alice(), GetAgentStats { id: id.clone() })
        .await
        .unwrap();

    assert_eq!(views.len(), 1);
    assert!(views[0].connected);
    assert_eq!(views[0].in_flight, 1);
    assert_eq!(view.app.id(), &id);
    assert!(view.connected);
    assert_eq!(stats.total, 3);
    assert_eq!(
        permissions.permissions()[1..],
        [
            Permission::ListAgents(org()),
            Permission::ReadApp(id.clone()),
            Permission::ReadAppStats(id),
        ]
    );
}

#[tokio::test]
async fn a_get_of_an_app_that_is_not_an_agent_is_not_found() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let err = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            GetAgent {
                id: AppId::new("ghost"),
            },
        )
        .await
        .err()
        .unwrap();

    assert!(matches!(err, DomainError::NotFound { .. }));
}

#[tokio::test]
async fn a_delete_disconnects_the_agent_removes_the_app_and_reloads_the_policies() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let id = lab.create().await.unwrap().app.id().clone();

    let deleted = lab
        .actions
        .run(&lab.uc, &alice(), DeleteAgent { id: id.clone() })
        .await
        .unwrap();

    assert_eq!(deleted.last_state(), &id);
    assert_eq!(lab.registry.disconnected(), vec![id.clone()]);
    assert!(lab.apps.rows.lock().unwrap().is_empty());
    assert!(lab.agents.rows.lock().unwrap().is_empty());
    assert_eq!(lab.policy.reloads(), 2);
    assert_eq!(permissions.permissions()[1], Permission::DeleteApp(id));
}

#[tokio::test]
async fn a_denied_delete_keeps_the_stream_and_the_row() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));
    let id = AppId::new("app-1");

    let err = lab
        .actions
        .run(&lab.uc, &alice(), DeleteAgent { id })
        .await
        .err()
        .unwrap();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.registry.disconnected().is_empty());
    assert_eq!(lab.policy.reloads(), 0);
}
