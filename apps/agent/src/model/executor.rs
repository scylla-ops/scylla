use crate::model::status::{EventKind, JobEvent, PipelineEvent, StageEvent, StatusSink, StepEvent};
use anyhow::Result;
use async_trait::async_trait;
use derive_builder::Builder;
use log::info;
use protocol::job::{Job, JobStage, JobStep};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub _stream: LogStream,
    pub chunk: String,
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
    pub status_sink: Arc<dyn StatusSink>,
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
}

impl<E: Executor> PipelineRunner<E> {
    pub async fn run_job(&self, pipeline: &Job) -> Result<()> {
        self.status_sink
            .on_event(PipelineEvent::Job(JobEvent {
                id: pipeline.id,
                name: pipeline.name.clone(),
                kind: EventKind::Running,
            }))
            .await;

        for stage in &pipeline.stages {
            self.status_sink
                .on_event(PipelineEvent::Stage(StageEvent {
                    id: stage.id,
                    kind: EventKind::Running,
                }))
                .await;

            match self.run_stage(stage).await {
                Ok(()) => {
                    self.status_sink
                        .on_event(PipelineEvent::Stage(StageEvent {
                            id: stage.id,
                            kind: EventKind::Succeeded,
                        }))
                        .await;
                }
                Err(e) => {
                    self.status_sink
                        .on_event(PipelineEvent::Job(JobEvent {
                            id: pipeline.id,
                            name: pipeline.name.clone(),
                            kind: EventKind::Failed,
                        }))
                        .await;
                    return Err(e);
                }
            }
        }

        self.status_sink
            .on_event(PipelineEvent::Job(JobEvent {
                id: pipeline.id,
                name: pipeline.name.clone(),
                kind: EventKind::Succeeded,
            }))
            .await;
        Ok(())
    }

    async fn run_stage(&self, stage: &JobStage) -> Result<()> {
        for step in &stage.steps {
            info!("Running step '{:?}'", step);
            self.status_sink
                .on_event(PipelineEvent::Step(StepEvent {
                    id: step.id,
                    kind: EventKind::Running,
                }))
                .await;

            let output = self
                .executor
                .run_step(ExecRequest {
                    step,
                    workdir: self.default_workdir.as_deref(),
                    env: Some(&self.default_env),
                    log_sink: self.log_sink.clone(),
                    status_sink: self.status_sink.clone(),
                })
                .await?;

            self.status_sink
                .on_event(PipelineEvent::Step(StepEvent {
                    id: step.id,
                    kind: if output.status.success() {
                        EventKind::Succeeded
                    } else {
                        EventKind::Failed
                    },
                }))
                .await;

            if !output.status.success() {
                self.status_sink
                    .on_event(PipelineEvent::Stage(StageEvent {
                        id: stage.id,
                        kind: EventKind::Failed,
                    }))
                    .await;
                return Err(anyhow::anyhow!(format!("Step {} failed", step.id)));
            }
        }
        Ok(())
    }
}
