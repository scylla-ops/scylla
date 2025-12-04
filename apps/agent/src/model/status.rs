use async_trait::async_trait;
use protocol::job::{JobData, JobEntry};
use protocol::toml;
use std::fmt::Debug;

#[async_trait]
pub trait StatusSink: Send + Sync + Debug {
    async fn on_event(&self, event: PipelineEvent);
}

#[derive(Debug, Clone)]
pub enum PipelineEvent {
    JobUpdate { job_entry: JobEntry },
}

impl TryFrom<PipelineEvent> for protocol::services::orchestrator::PipelineStatuUpdate {
    type Error = toml::ser::Error;

    fn try_from(value: PipelineEvent) -> Result<Self, Self::Error> {
        match value {
            PipelineEvent::JobUpdate { job_entry } => {
                let job_data = JobData::from(job_entry.job);
                let job_data_toml = toml::to_string(&job_data)?;
                Ok(Self {
                    job_id: job_entry.id,
                    job_data_toml,
                })
            }
        }
    }
}
