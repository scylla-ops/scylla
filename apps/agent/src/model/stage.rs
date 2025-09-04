use crate::model::step::Step;
use derive_more::Constructor;
use protocol::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct Stage {
    pub name: String,
    pub steps: Vec<Step>,
}
