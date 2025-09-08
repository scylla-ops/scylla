use crate::shell::Shell;
use crate::{Deserialize, Serialize};
use derive_builder::Builder;
use derive_more::Constructor;

#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Builder)]
pub struct Pipeline {
    pub name: String,
    pub stages: Vec<PStage>,
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
