use crate::pipeline::{PStage, PStep, Pipeline};
use crate::shell::Shell;
use crate::{Deserialize, Serialize};
use derive_builder::Builder;
use derive_more::Constructor;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Builder)]
pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub stages: Vec<JobStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Builder)]
pub struct JobStage {
    pub id: Uuid,
    pub name: String,
    pub steps: Vec<JobStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Builder)]
pub struct JobStep {
    pub id: Uuid,
    pub name: String,
    pub shell: Shell,
    pub command: String,
    pub args: Vec<String>,
}

impl From<Pipeline> for Job {
    fn from(value: Pipeline) -> Self {
        Job {
            id: Uuid::new_v4(),
            name: value.name,
            stages: value.stages.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PStage> for JobStage {
    fn from(value: PStage) -> Self {
        JobStage {
            id: Uuid::new_v4(),
            name: value.name,
            steps: value.steps.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PStep> for JobStep {
    fn from(value: PStep) -> Self {
        JobStep {
            id: Uuid::new_v4(),
            name: value.name,
            shell: value.shell,
            command: value.command,
            args: value.args,
        }
    }
}
