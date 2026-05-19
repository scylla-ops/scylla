mod agent_listener;
mod error;
mod log_listener;
mod status_listener;

pub use error::ListenerError;

use scylla_core::application::{
    AgentUseCases, JobLogUseCases, JobUseCases, PermissionService,
};
use scylla_core::infrastructure::{PgAgentRepository, PgJobLogRepository, PgJobRepository};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tonic::transport::Channel;
use tracing::info;

/// Subset of application services the recorder needs. Built once by the
/// composition root (control-plane) and passed in. Generic over the concrete
/// `PermissionService` so the recorder library does not have to know about
/// Cedar (or whatever ships next).
pub struct RecorderServices<PS: PermissionService> {
    pub job_uc: Arc<JobUseCases<PgJobRepository, PS>>,
    pub job_log_uc: Arc<JobLogUseCases<PgJobLogRepository, PS>>,
    pub agent_uc: Arc<AgentUseCases<PgAgentRepository, PS>>,
}

impl<PS: PermissionService> Clone for RecorderServices<PS> {
    fn clone(&self) -> Self {
        Self {
            job_uc: self.job_uc.clone(),
            job_log_uc: self.job_log_uc.clone(),
            agent_uc: self.agent_uc.clone(),
        }
    }
}

/// Spawn the 4 broker subscribers (status, logs, agent heartbeat, agent shutdown).
///
/// Each listener owns its own broker subscription and reconnects on failure;
/// the returned join handles complete only when their stream closes
/// permanently (e.g. process shutdown). Every listener invokes its use case
/// with `CallerContext::Service(ServiceIdentity::recorder())`.
pub fn spawn_listeners<PS: PermissionService + 'static>(
    broker: Channel,
    services: RecorderServices<PS>,
) -> Vec<JoinHandle<()>> {
    info!(service = "recorder", "spawning event listeners");
    vec![
        tokio::spawn(status_listener::run(broker.clone(), services.job_uc)),
        tokio::spawn(log_listener::run(broker.clone(), services.job_log_uc)),
        tokio::spawn(agent_listener::run_heartbeat(
            broker.clone(),
            services.agent_uc.clone(),
        )),
        tokio::spawn(agent_listener::run_shutdown(broker, services.agent_uc)),
    ]
}
