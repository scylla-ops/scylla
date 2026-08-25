use crate::application::HashService;
use crate::application::agent::dispatch::JobDispatch;
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::application::agent::repository::{AgentRepository, AgentStats};
use crate::application::app::repository::AppRepository;
use crate::application::authz::grant::{Grant, ORGANIZATION_AGENT_ROLE, Principal, Scope};
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::domain::agent::{Agent, AgentHost};
use crate::domain::app::{App, AppCredential};
use crate::domain::app::{AppName, AppSecret, AppSecretLabel};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, OrganizationId, PipelineId};
use crate::domain::permission::Permission;
use crate::domain::role::RoleName;
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{instrument, warn};

/// Label given to an agent's initial secret, created alongside the agent.
const DEFAULT_SECRET_LABEL: &str = "default";

/// Result of trying to place a job on an agent.
pub enum DispatchOutcome {
    Dispatched(AppId),
    NoAgentAvailable,
}

/// Chooses a connected agent App and hands it a job. Eligibility is pure Cedar
/// (`check(App, ExecuteJob(pipeline))` — the agent holds a grant covering the
/// pipeline's org/project); among the eligible agents, jobs are spread
/// round-robin so two equally-authorized agents share the load instead of the
/// first one taking every job. No ad-hoc routing.
pub struct DispatchUseCases<W: AgentDispatch, PS: PermissionService> {
    registry: Arc<W>,
    permission_service: Arc<PS>,
    /// Rotating start offset into `connected()`: each dispatch begins its scan
    /// one agent further along, so consecutive jobs land on different agents.
    /// Wraps freely — only the offset's relative position matters.
    next: AtomicUsize,
}

impl<W: AgentDispatch, PS: PermissionService> DispatchUseCases<W, PS> {
    #[must_use]
    pub fn new(registry: Arc<W>, permission_service: Arc<PS>) -> Self {
        Self {
            registry,
            permission_service,
            next: AtomicUsize::new(0),
        }
    }

    /// Dispatch a pipeline's job to a connected agent authorized to execute it,
    /// rotating the starting point each call so eligible agents share the load.
    /// Best-effort: if none is connected+authorized the job stays pending
    /// (`NoAgentAvailable`) rather than failing the run.
    #[instrument(skip_all, fields(pipeline_id = %pipeline_id, job_id = %dispatch.job_id))]
    pub async fn dispatch_job(
        &self,
        pipeline_id: &PipelineId,
        dispatch: &JobDispatch,
    ) -> DomainResult<DispatchOutcome> {
        let agents = self.registry.connected();
        if agents.is_empty() {
            warn!(pipeline_id = %pipeline_id, "no connected agent; job left pending");
            return Ok(DispatchOutcome::NoAgentAvailable);
        }

        // Least-loaded: try agents idlest-first (fewest in-flight jobs), and
        // rotate among equally-idle agents by a per-dispatch offset so they
        // still take turns. The first *eligible* agent in that order wins, so a
        // busy or unauthorized agent is skipped for an idle authorized one.
        let start = self.next.fetch_add(1, Ordering::Relaxed);
        let n = agents.len();
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by_key(|&i| {
            (
                self.registry.in_flight(&agents[i]),
                start.wrapping_add(i) % n,
            )
        });
        for i in order {
            let app_id = &agents[i];
            let caller = CallerContext::App(app_id.clone());
            match self
                .permission_service
                .check(&caller, Permission::ExecuteJob(pipeline_id.clone()))
                .await
            {
                Ok(()) => match self.registry.dispatch(app_id, dispatch).await {
                    Ok(()) => return Ok(DispatchOutcome::Dispatched(app_id.clone())),
                    // The agent disconnected since `connected()` snapshotted
                    // (check-then-act race) or its queue is gone — try the next
                    // candidate rather than failing the whole run.
                    Err(e) => {
                        warn!(app_id = %app_id, error = %e, "dispatch to agent failed; trying next");
                    }
                },
                // Not authorized for this pipeline — expected; try the next agent.
                Err(DomainError::Forbidden(_)) => {}
                // A real failure (e.g. authz DB blip): don't silently treat it as
                // a clean deny — log it, then still try the remaining agents.
                Err(e) => {
                    warn!(app_id = %app_id, error = %e, "authz check errored during dispatch; skipping agent");
                }
            }
        }
        warn!(
            pipeline_id = %pipeline_id,
            "no connected agent authorized to execute pipeline; job left pending"
        );
        Ok(DispatchOutcome::NoAgentAvailable)
    }
}

/// What a successful agent `create` returns: the persisted app backing the
/// agent plus its plaintext secret, shown exactly once and never stored.
pub struct CreatedAgent {
    pub app: App,
    pub secret: AppSecret,
}

/// Read model for an agent: its backing app identity, live connection state
/// (from the in-memory registry), and durable last-seen. `connected` and the
/// registry are infra state, so this is a use-case DTO, not a domain entity.
pub struct AgentView {
    pub app: App,
    pub connected: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub in_flight: usize,
    pub host: Option<AgentHost>,
}

/// Org-scoped management + introspection of Agents (specialized apps that run
/// jobs). Creating an agent provisions an app, its `agents` row and an agent
/// grant on the org, then reloads the policy set so the grant is live at once.
/// Every method is Cedar-gated.
#[derive(Constructor)]
pub struct AgentUseCases<A, W, H, PC, PS>
where
    A: AppRepository,
    W: AgentRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
{
    app_repo: Arc<A>,
    agent_repo: Arc<W>,
    hash_service: Arc<H>,
    policy_control: Arc<PC>,
    permission_service: Arc<PS>,
    registry: Arc<dyn AgentDispatch>,
}

impl<A, W, H, PC, PS> AgentUseCases<A, W, H, PC, PS>
where
    A: AppRepository,
    W: AgentRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
{
    #[instrument(skip_all, fields(org_id = %organization_id, name = %name))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        organization_id: OrganizationId,
        name: AppName,
    ) -> DomainResult<CreatedAgent> {
        self.permission_service
            .check(caller, Permission::CreateAgent(organization_id.clone()))
            .await?;

        let secret = crate::application::app::mint_app_secret();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let app = App::create(organization_id.clone(), name);
        let credential = AppCredential::create(
            app.id().clone(),
            AppSecretLabel::new(DEFAULT_SECRET_LABEL)?,
            secret_hash,
        );
        let agent = Agent::create(app.id().clone());

        // The agent pulls and executes jobs across its org's pipelines via a
        // scoped agent grant — the same role a plain app no longer gets. The
        // initial secret is written in the same tx so the agent can authenticate.
        let grant = Grant::new(
            Principal::App(app.id().clone()),
            RoleName::new(ORGANIZATION_AGENT_ROLE)?,
            Scope::Organization(organization_id),
        );
        self.app_repo
            .provision_agent(&app, &credential, &agent, &grant)
            .await?;
        self.policy_control.reload().await?;

        Ok(CreatedAgent { app, secret })
    }

    #[instrument(skip_all, fields(org_id = %organization_id))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        organization_id: OrganizationId,
    ) -> DomainResult<Vec<AgentView>> {
        self.permission_service
            .check(caller, Permission::ListAgents(organization_id.clone()))
            .await?;

        let agents = self
            .agent_repo
            .list_by_organization(&organization_id)
            .await?;
        let connected: HashSet<String> = self
            .registry
            .connected()
            .into_iter()
            .map(|id| id.as_str().to_string())
            .collect();

        let mut views = Vec::with_capacity(agents.len());
        for agent in &agents {
            let app = self.app_repo.find_by_id(agent.app_id()).await?;
            let is_connected = connected.contains(app.id().as_str());
            let in_flight = self.registry.in_flight(agent.app_id());
            views.push(AgentView {
                app,
                connected: is_connected,
                last_seen: agent.last_seen(),
                in_flight,
                host: agent.host().cloned(),
            });
        }
        Ok(views)
    }

    #[instrument(skip_all, fields(app_id = %app_id))]
    pub async fn get(&self, caller: &CallerContext, app_id: AppId) -> DomainResult<AgentView> {
        self.permission_service
            .check(caller, Permission::ReadApp(app_id.clone()))
            .await?;

        let agent = self.agent_repo.find_by_app_id(&app_id).await?;
        let app = self.app_repo.find_by_id(&app_id).await?;
        let connected = self
            .registry
            .connected()
            .iter()
            .any(|id| id.as_str() == app_id.as_str());
        Ok(AgentView {
            app,
            connected,
            last_seen: agent.last_seen(),
            in_flight: self.registry.in_flight(&app_id),
            host: agent.host().cloned(),
        })
    }

    #[instrument(skip_all, fields(app_id = %app_id))]
    pub async fn stats(&self, caller: &CallerContext, app_id: AppId) -> DomainResult<AgentStats> {
        self.permission_service
            .check(caller, Permission::ReadAppStats(app_id.clone()))
            .await?;
        self.agent_repo.agent_stats(&app_id).await
    }

    #[instrument(skip_all, fields(app_id = %app_id))]
    pub async fn delete(&self, caller: &CallerContext, app_id: AppId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeleteApp(app_id.clone()))
            .await?;
        // Drop the live stream first so a removed agent stops at once; the app
        // delete cascades the agents row + grants and nulls jobs.agent_app_id.
        self.registry.disconnect(&app_id);
        self.app_repo.delete(&app_id).await?;
        self.policy_control.reload().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct StubRegistry {
        connected: Vec<AppId>,
        dispatched: Mutex<Vec<String>>,
        loads: Mutex<HashMap<String, usize>>,
    }

    impl StubRegistry {
        fn new(connected: Vec<AppId>) -> Self {
            Self {
                connected,
                dispatched: Mutex::new(vec![]),
                loads: Mutex::new(HashMap::new()),
            }
        }
        fn with_loads(connected: Vec<AppId>, loads: HashMap<String, usize>) -> Self {
            Self {
                connected,
                dispatched: Mutex::new(vec![]),
                loads: Mutex::new(loads),
            }
        }
    }

    #[async_trait]
    impl AgentDispatch for StubRegistry {
        fn connected(&self) -> Vec<AppId> {
            self.connected.clone()
        }
        async fn dispatch(&self, app_id: &AppId, _dispatch: &JobDispatch) -> DomainResult<()> {
            self.dispatched
                .lock()
                .unwrap()
                .push(app_id.as_str().to_string());
            *self
                .loads
                .lock()
                .unwrap()
                .entry(app_id.as_str().to_string())
                .or_insert(0) += 1;
            Ok(())
        }
        fn disconnect(&self, _app_id: &AppId) {}
        fn in_flight(&self, app_id: &AppId) -> usize {
            *self
                .loads
                .lock()
                .unwrap()
                .get(app_id.as_str())
                .unwrap_or(&0)
        }
        fn release(&self, app_id: &AppId) {
            if let Some(v) = self.loads.lock().unwrap().get_mut(app_id.as_str()) {
                *v = v.saturating_sub(1);
            }
        }
    }

    struct StubPerms {
        allowed: &'static str,
    }

    #[async_trait]
    impl PermissionService for StubPerms {
        async fn check(&self, caller: &CallerContext, _perm: Permission) -> DomainResult<()> {
            if matches!(caller, CallerContext::App(id) if id.as_str() == self.allowed) {
                Ok(())
            } else {
                Err(DomainError::forbidden("not the allowed agent"))
            }
        }
    }

    struct StubPermsAll;

    #[async_trait]
    impl PermissionService for StubPermsAll {
        async fn check(&self, _caller: &CallerContext, _perm: Permission) -> DomainResult<()> {
            Ok(())
        }
    }

    fn dispatch() -> JobDispatch {
        JobDispatch {
            job_id: "j1".to_string(),
            pipeline_id: "pl1".to_string(),
            nodes: vec![],
        }
    }

    #[tokio::test]
    async fn dispatches_to_first_authorized_connected_agent() {
        let registry = Arc::new(StubRegistry::new(vec![
            AppId::new("app-unauthorized"),
            AppId::new("app-ok"),
        ]));
        let uc = DispatchUseCases::new(registry.clone(), Arc::new(StubPerms { allowed: "app-ok" }));

        let outcome = uc
            .dispatch_job(&PipelineId::new("pl1"), &dispatch())
            .await
            .unwrap();

        assert!(matches!(outcome, DispatchOutcome::Dispatched(id) if id.as_str() == "app-ok"));
        assert_eq!(registry.dispatched.lock().unwrap().as_slice(), ["app-ok"]);
    }

    #[tokio::test]
    async fn spreads_jobs_round_robin_across_authorized_agents() {
        // Two equally-authorized connected agents. Before the round-robin fix
        // every job went to the first one and the second starved; now four
        // dispatches must split two-and-two.
        let registry = Arc::new(StubRegistry::new(vec![
            AppId::new("app-a"),
            AppId::new("app-b"),
        ]));
        let uc = DispatchUseCases::new(registry.clone(), Arc::new(StubPermsAll));

        for _ in 0..4 {
            uc.dispatch_job(&PipelineId::new("pl1"), &dispatch())
                .await
                .unwrap();
        }

        let dispatched = registry.dispatched.lock().unwrap().clone();
        assert_eq!(dispatched.len(), 4);
        assert_eq!(
            dispatched.iter().filter(|x| x.as_str() == "app-a").count(),
            2,
            "app-a should get half the jobs"
        );
        assert_eq!(
            dispatched.iter().filter(|x| x.as_str() == "app-b").count(),
            2,
            "app-b should get half the jobs (no longer starved)"
        );
    }

    #[tokio::test]
    async fn picks_least_loaded_eligible_agent() {
        // app-busy already has 3 in-flight jobs; app-idle has none. The next job
        // must go to the idle one regardless of connection order.
        let mut loads = HashMap::new();
        loads.insert("app-busy".to_string(), 3);
        let registry = Arc::new(StubRegistry::with_loads(
            vec![AppId::new("app-busy"), AppId::new("app-idle")],
            loads,
        ));
        let uc = DispatchUseCases::new(registry.clone(), Arc::new(StubPermsAll));

        uc.dispatch_job(&PipelineId::new("pl1"), &dispatch())
            .await
            .unwrap();

        assert_eq!(registry.dispatched.lock().unwrap().as_slice(), ["app-idle"]);
    }

    #[tokio::test]
    async fn no_agent_when_none_authorized() {
        let registry = Arc::new(StubRegistry::new(vec![AppId::new("app-x")]));
        let uc = DispatchUseCases::new(registry, Arc::new(StubPerms { allowed: "nobody" }));

        let outcome = uc
            .dispatch_job(&PipelineId::new("pl1"), &dispatch())
            .await
            .unwrap();

        assert!(matches!(outcome, DispatchOutcome::NoAgentAvailable));
    }
}
