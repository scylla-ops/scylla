use crate::model::stage::Stage;
use derive_more::Constructor;
use protocol::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct Pipeline {
    pub name: String,
    pub stages: Vec<Stage>,
}
