use crate::application::caller::CallerContext;
use crate::application::permission::service::PermissionService;
use crate::application::worker::dispatch_port::WorkerDispatch;
use crate::domain::entities::{AppId, PipelineId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::pipeline::JobDispatch;
use derive_more::Constructor;
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
