use crate::model::status::{PipelineEvent, StatusSink};
use anyhow::Result;
use async_trait::async_trait;
use derive_builder::Builder;
use protocol::job::{ExecutionStatus, JobEntry, JobStep};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum LogStream {
    Stdout,
    _Stderr,
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub _stream: LogStream,
    pub _chunk: String,
}

#[async_trait]
pub trait LogSink: Send + Sync + Debug {
    async fn on_log_chunk(&self, ev: LogEvent);
}

#[derive(Debug, Clone)]
pub struct ExecRequest<'a> {
    pub step: &'a JobStep,
    pub workdir: Option<&'a Path>,
    pub env: Option<&'a HashMap<String, String>>,
    pub log_sink: Option<Arc<dyn LogSink>>,
}

#[derive(Debug, Clone)]
pub struct ExecOutput {
    pub status: ExitStatus,
}

#[async_trait]
pub trait Executor: Send + Sync {
    async fn run_step(&self, req: ExecRequest<'_>) -> Result<ExecOutput>;
}

#[derive(Builder)]
pub struct PipelineRunner<E: Executor> {
    executor: E,
    default_workdir: Option<PathBuf>,
    default_env: HashMap<String, String>,
    log_sink: Option<Arc<dyn LogSink>>,
    status_sink: Arc<dyn StatusSink>,
    job: JobEntry,
}

impl<E: Executor> PipelineRunner<E> {
    /// Modifie + émet en une seule op
    async fn update<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut JobEntry),
    {
        f(&mut self.job);
        self.status_sink
            .on_event(PipelineEvent::JobUpdate {
                job_entry: self.job.clone(),
            })
            .await;
        Ok(())
    }

    pub async fn run_job(&mut self) -> Result<()> {
        self.update(|j| j.job.state = ExecutionStatus::Running)
            .await?;

        for stage_idx in 0..self.job.job.stages.len() {
            self.update(|j| j.job.stages[stage_idx].state = ExecutionStatus::Running)
                .await?;

            if let Err(e) = self.run_stage_at(stage_idx).await {
                self.update(|j| {
                    j.job.stages[stage_idx].state = ExecutionStatus::Failed;
                    j.job.state = ExecutionStatus::Failed;
                })
                .await?;
                return Err(e);
            }

            self.update(|j| j.job.stages[stage_idx].state = ExecutionStatus::Succeeded)
                .await?;
        }

        self.update(|j| j.job.state = ExecutionStatus::Succeeded)
            .await?;
        Ok(())
    }

    async fn run_stage_at(&mut self, stage_idx: usize) -> Result<()> {
        for step_idx in 0..self.job.job.stages[stage_idx].steps.len() {
            self.update(|j| {
                j.job.stages[stage_idx].steps[step_idx].state = ExecutionStatus::Running
            })
            .await?;

            let step = &self.job.job.stages[stage_idx].steps[step_idx];
            let output = self
                .executor
                .run_step(ExecRequest {
                    step,
                    workdir: self.default_workdir.as_deref(),
                    env: Some(&self.default_env),
                    log_sink: self.log_sink.clone(),
                })
                .await?;

            let step_uuid = step.uuid;

            if output.status.success() {
                self.update(|j| {
                    j.job.stages[stage_idx].steps[step_idx].state = ExecutionStatus::Succeeded
                })
                .await?;
            } else {
                self.update(|j| {
                    j.job.stages[stage_idx].steps[step_idx].state = ExecutionStatus::Failed
                })
                .await?;
                return Err(anyhow::anyhow!(format!("Step {} failed", step_uuid)));
            }
        }
        Ok(())
    }
}
