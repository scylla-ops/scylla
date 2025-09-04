use crate::model::shell::Shell;
use derive_more::Constructor;
use protocol::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct Step {
    pub name: String,
    pub shell: Shell,
    pub command: String,
    pub args: Vec<String>,
}
