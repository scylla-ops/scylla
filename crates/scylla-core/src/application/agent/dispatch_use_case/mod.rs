//! The placement of a stored job on a connected agent. `place` runs in the `commit` closure of
//! `RunPipeline`, `RunPipelineWithInputs` and `DispatchPendingJobs`, after the job row exists.

pub mod commands;

pub use commands::DispatchPendingJobs;

use crate::application::agent::dispatch::JobDispatch;
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::application::{JobRepository, PipelineRepository, SecretResolver};
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainError;
use crate::domain::ids::{AppId, PipelineId};
use crate::domain::job::Job;
use crate::domain::permission::Permission;
use scylla_auth::authz::PermissionService;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{info, warn};

pub enum DispatchOutcome {
    Dispatched(AppId),
    NoAgentAvailable,
}

/// The stage runner of `DispatchPendingJobs`. `permission_service` asks for `ExecuteJob` only to
/// choose an agent, never to refuse.
pub struct DispatchUseCases {
    registry: Arc<dyn AgentDispatch>,
    permission_service: Arc<dyn PermissionService>,
    job_repo: Arc<dyn JobRepository>,
    pipeline_repo: Arc<dyn PipelineRepository>,
    secret_resolver: Arc<dyn SecretResolver>,
    next: AtomicUsize,
}

impl DispatchUseCases {
    #[must_use]
    pub fn new(
        registry: Arc<dyn AgentDispatch>,
        permission_service: Arc<dyn PermissionService>,
        job_repo: Arc<dyn JobRepository>,
        pipeline_repo: Arc<dyn PipelineRepository>,
        secret_resolver: Arc<dyn SecretResolver>,
    ) -> Self {
        Self {
            registry,
            permission_service,
            job_repo,
            pipeline_repo,
            secret_resolver,
            next: AtomicUsize::new(0),
        }
    }

    /// Best-effort once the job exists: a failed attribution is logged, and the job runs anyway.
    pub(crate) async fn place(&self, job: &mut Job, dispatch: &JobDispatch) -> DispatchOutcome {
        let outcome = self.dispatch_job(job.pipeline_id(), dispatch).await;
        if let DispatchOutcome::Dispatched(app_id) = &outcome {
            info!(job_id = %job.id(), %app_id, "job dispatched to agent");
            match self.job_repo.set_agent(job.id(), app_id).await {
                Ok(()) => job.assign_agent(app_id.clone()),
                Err(e) => {
                    warn!(job_id = %job.id(), %app_id, error = %e, "failed to record job agent attribution");
                }
            }
        }
        outcome
    }

    async fn dispatch_job(
        &self,
        pipeline_id: &PipelineId,
        dispatch: &JobDispatch,
    ) -> DispatchOutcome {
        let agents = self.registry.connected();
        if agents.is_empty() {
            warn!(pipeline_id = %pipeline_id, "no connected agent; job left pending");
            return DispatchOutcome::NoAgentAvailable;
        }

        // Idlest first, then rotate among equals so eligible agents take turns.
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
                    Ok(()) => return DispatchOutcome::Dispatched(app_id.clone()),
                    // Disconnected since `connected()` was snapshotted: try the next one.
                    Err(e) => {
                        warn!(app_id = %app_id, error = %e, "dispatch to agent failed; trying next");
                    }
                },
                Err(DomainError::Forbidden(_)) => {}
                Err(e) => {
                    warn!(app_id = %app_id, error = %e, "authz check errored during dispatch; skipping agent");
                }
            }
        }
        warn!(
            pipeline_id = %pipeline_id,
            "no connected agent authorized to execute pipeline; job left pending"
        );
        DispatchOutcome::NoAgentAvailable
    }
}

#[cfg(test)]
mod tests;
