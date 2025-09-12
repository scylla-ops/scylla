use anyhow::Result;
use async_trait::async_trait;
use protocol::pipeline::{PStage, PStep, Pipeline};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct ExecRequest<'a> {
    pub step: &'a PStep,
    pub workdir: Option<&'a Path>,
    pub env: Option<&'a HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct ExecOutput {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[async_trait]
pub trait Executor: Send + Sync {
    async fn run_step(&self, req: ExecRequest<'_>) -> Result<ExecOutput>;
}

pub struct PipelineRunner<E: Executor> {
    executor: E,
    default_workdir: Option<PathBuf>,
    default_env: HashMap<String, String>,
}

impl<E: Executor> PipelineRunner<E> {
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            default_workdir: None,
            default_env: HashMap::new(),
        }
    }

    pub fn with_workdir(mut self, workdir: impl Into<PathBuf>) -> Self {
        self.default_workdir = Some(workdir.into());
        self
    }

    pub fn with_env_var(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.default_env.insert(key.into(), val.into());
        self
    }

    pub async fn run_pipeline(&self, pipeline: &Pipeline) -> Result<()> {
        for stage in &pipeline.stages {
            self.run_stage(stage).await?;
        }
        Ok(())
    }

    async fn run_stage(&self, stage: &PStage) -> Result<()> {
        for step in &stage.steps {
            println!("Running step '{:?}'", step);

            let output = self
                .executor
                .run_step(ExecRequest {
                    step,
                    workdir: self.default_workdir.as_deref(),
                    env: Some(&self.default_env),
                })
                .await?;

            if !(output.status_code == 0) {
                anyhow::bail!(
                    "Échec du step '{}' (shell={:?}) code={}.\nstdout:\n{}\nstderr:\n{}",
                    step.name,
                    step.shell,
                    output.status_code,
                    output.stdout,
                    output.stderr
                );
            }

            debug!("output = {:#?}", output);
        }
        Ok(())
    }
}
