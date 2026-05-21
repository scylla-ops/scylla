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
