use crate::shell::Shell;
use crate::{Deserialize, Serialize};
use derive_builder::Builder;
use derive_more::Constructor;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Builder)]
pub struct Pipeline {
    pub name: String,
    pub stages: Vec<PStage>,
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Failed to serialize pipeline to TOML: {0}")]
    Serialization(#[from] toml::ser::Error),
}

impl Pipeline {
    pub fn as_bytes(&self) -> Result<Vec<u8>, PipelineError> {
        let toml_string = toml::to_string(self)?;
        Ok(toml_string.into_bytes())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Builder)]
pub struct PStage {
    pub name: String,
    pub steps: Vec<PStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Builder)]
pub struct PStep {
    pub name: String,
    pub shell: Shell,
    pub command: String,
    pub args: Vec<String>,
}
