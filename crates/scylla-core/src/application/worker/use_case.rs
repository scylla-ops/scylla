use crate::application::HashService;
use crate::application::app::repository::AppRepository;
use crate::application::caller::CallerContext;
use crate::application::permission::grant::{Grant, GrantPrincipal, GrantScope, WORKER_ROLE};
use crate::application::permission::policy::PolicyControl;
use crate::application::permission::service::PermissionService;
use crate::application::worker::dispatch_port::WorkerDispatch;
use crate::application::worker::repository::{WorkerRepository, WorkerStats};
use crate::domain::entities::{App, AppId, OrganizationId, PipelineId, Worker};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::app::{AppName, AppSecret};
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::pipeline::JobDispatch;
use crate::domain::value_objects::role::name::RoleName;
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{instrument, warn};

/// Result of trying to place a job on a worker.
pub enum DispatchOutcome {
    Dispatched(AppId),
    NoWorkerAvailable,
}

/// Chooses a connected worker App and hands it a job. Selection is pure Cedar:
/// a worker is eligible iff `check(App, ExecuteJob(pipeline))` passes — i.e. it
/// holds a worker grant covering the pipeline's org/project. No ad-hoc routing.
#[derive(Constructor)]
pub struct DispatchUseCases<W: WorkerDispatch, PS: PermissionService> {
    registry: Arc<W>,
    permission_service: Arc<PS>,
}

impl<W: WorkerDispatch, PS: PermissionService> DispatchUseCases<W, PS> {
    /// Dispatch a pipeline's job to the first connected worker authorized to
    /// execute it. Best-effort: if none is connected+authorized the job stays
    /// pending (`NoWorkerAvailable`) rather than failing the run.
    #[instrument(skip(self, dispatch), fields(pipeline_id = %pipeline_id, job_id = %dispatch.job_id))]
    pub async fn dispatch_job(
        &self,
        pipeline_id: &PipelineId,
        dispatch: &JobDispatch,
    ) -> DomainResult<DispatchOutcome> {
        for app_id in self.registry.connected() {
            let caller = CallerContext::App(app_id.clone());
            let authorized = self
                .permission_service
                .check(&caller, Permission::ExecuteJob(pipeline_id.clone()))
                .await
                .unwrap_or(false);
            if authorized {
                self.registry.dispatch(&app_id, dispatch).await?;
                return Ok(DispatchOutcome::Dispatched(app_id));
            }
        }
        warn!(
            pipeline_id = %pipeline_id,
            "no connected worker authorized to execute pipeline; job left pending"
        );
        Ok(DispatchOutcome::NoWorkerAvailable)
    }
}

/// What a successful worker `create` returns: the persisted app backing the
/// worker plus its plaintext secret, shown exactly once and never stored.
pub struct CreatedWorker {
    pub app: App,
    pub secret: AppSecret,
}

/// Read model for a worker: its backing app identity, live connection state
/// (from the in-memory registry), and durable last-seen. `connected` and the
/// registry are infra state, so this is a use-case DTO, not a domain entity.
pub struct WorkerView {
    pub app: App,
    pub connected: bool,
    pub last_seen: Option<DateTime<Utc>>,
}

/// Org-scoped management + introspection of Workers (specialized apps that run
/// jobs). Creating a worker provisions an app, its `workers` row and a worker
/// grant on the org, then reloads the policy set so the grant is live at once.
/// Every method is Cedar-gated.
#[derive(Constructor)]
pub struct WorkerUseCases<A, W, H, PC, PS>
where
    A: AppRepository,
    W: WorkerRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
{
    app_repo: Arc<A>,
    worker_repo: Arc<W>,
    hash_service: Arc<H>,
    policy_control: Arc<PC>,
    permission_service: Arc<PS>,
    registry: Arc<dyn WorkerDispatch>,
}

impl<A, W, H, PC, PS> WorkerUseCases<A, W, H, PC, PS>
where
    A: AppRepository,
    W: WorkerRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
{
    #[instrument(skip(self, caller), fields(org_id = %organization_id, name = %name))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        organization_id: OrganizationId,
        name: AppName,
    ) -> DomainResult<CreatedWorker> {
        self.permission_service
            .check(caller, Permission::CreateWorker(organization_id.clone()))
            .await?;

        let secret = AppSecret::generate();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let app = App::create(organization_id.clone(), name, secret_hash);
        let worker = Worker::create(app.id().clone());

        // The worker pulls and executes jobs across its org's pipelines via a
        // scoped worker grant — the same role a plain app no longer gets.
        let grant = Grant::new(
            GrantPrincipal::App(app.id().clone()),
            RoleName::new(WORKER_ROLE)?,
            GrantScope::Organization(organization_id),
        );
        self.app_repo.provision_worker(&app, &worker, &grant).await?;
        self.policy_control.reload().await?;

        Ok(CreatedWorker { app, secret })
    }

    #[instrument(skip(self, caller), fields(org_id = %organization_id))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        organization_id: OrganizationId,
    ) -> DomainResult<Vec<WorkerView>> {
        self.permission_service
            .check(caller, Permission::ListWorkers(organization_id.clone()))
            .await?;

        let workers = self
            .worker_repo
            .list_by_organization(&organization_id)
            .await?;
        let connected: HashSet<String> = self
            .registry
            .connected()
            .into_iter()
            .map(|id| id.as_str().to_string())
            .collect();

        let mut views = Vec::with_capacity(workers.len());
        for worker in &workers {
            let app = self.app_repo.find_by_id(worker.app_id()).await?;
            let is_connected = connected.contains(app.id().as_str());
            views.push(WorkerView {
                app,
                connected: is_connected,
                last_seen: worker.last_seen(),
            });
        }
        Ok(views)
    }

    #[instrument(skip(self, caller), fields(app_id = %app_id))]
    pub async fn get(&self, caller: &CallerContext, app_id: AppId) -> DomainResult<WorkerView> {
        self.permission_service
            .check(caller, Permission::ReadWorker(app_id.clone()))
            .await?;

        let worker = self.worker_repo.find_by_app_id(&app_id).await?;
        let app = self.app_repo.find_by_id(&app_id).await?;
        let connected = self
            .registry
            .connected()
            .iter()
            .any(|id| id.as_str() == app_id.as_str());
        Ok(WorkerView {
            app,
            connected,
            last_seen: worker.last_seen(),
        })
    }

    #[instrument(skip(self, caller), fields(app_id = %app_id))]
    pub async fn stats(&self, caller: &CallerContext, app_id: AppId) -> DomainResult<WorkerStats> {
        self.permission_service
            .check(caller, Permission::ReadWorkerStats(app_id.clone()))
            .await?;
        self.worker_repo.worker_stats(&app_id).await
    }

    #[instrument(skip(self, caller), fields(app_id = %app_id))]
    pub async fn delete(&self, caller: &CallerContext, app_id: AppId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeleteWorker(app_id.clone()))
            .await?;
        // Drop the live stream first so a removed worker stops at once; the app
        // delete cascades the workers row + grants and nulls jobs.worker_app_id.
        self.registry.disconnect(&app_id);
        self.app_repo.delete(&app_id).await?;
        self.policy_control.reload().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct StubRegistry {
        connected: Vec<AppId>,
        dispatched: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl WorkerDispatch for StubRegistry {
        fn connected(&self) -> Vec<AppId> {
            self.connected.clone()
        }
        async fn dispatch(&self, app_id: &AppId, _dispatch: &JobDispatch) -> DomainResult<()> {
            self.dispatched
                .lock()
                .unwrap()
                .push(app_id.as_str().to_string());
            Ok(())
        }
        fn disconnect(&self, _app_id: &AppId) {}
    }

    struct StubPerms {
        allowed: &'static str,
    }

    #[async_trait]
    impl PermissionService for StubPerms {
        async fn check(&self, caller: &CallerContext, _perm: Permission) -> DomainResult<bool> {
            Ok(matches!(caller, CallerContext::App(id) if id.as_str() == self.allowed))
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
    async fn dispatches_to_first_authorized_connected_worker() {
        let registry = Arc::new(StubRegistry {
            connected: vec![AppId::new("app-unauthorized"), AppId::new("app-ok")],
            dispatched: Mutex::new(vec![]),
        });
        let uc = DispatchUseCases::new(registry.clone(), Arc::new(StubPerms { allowed: "app-ok" }));

        let outcome = uc
            .dispatch_job(&PipelineId::new("pl1"), &dispatch())
            .await
            .unwrap();

        assert!(matches!(outcome, DispatchOutcome::Dispatched(id) if id.as_str() == "app-ok"));
        assert_eq!(registry.dispatched.lock().unwrap().as_slice(), ["app-ok"]);
    }

    #[tokio::test]
    async fn no_worker_when_none_authorized() {
        let registry = Arc::new(StubRegistry {
            connected: vec![AppId::new("app-x")],
            dispatched: Mutex::new(vec![]),
        });
        let uc = DispatchUseCases::new(registry, Arc::new(StubPerms { allowed: "nobody" }));

        let outcome = uc
            .dispatch_job(&PipelineId::new("pl1"), &dispatch())
            .await
            .unwrap();

        assert!(matches!(outcome, DispatchOutcome::NoWorkerAvailable));
    }
}
