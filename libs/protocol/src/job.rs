use crate::pipeline::{PStage, PStep, Pipeline};
use derive_more::Constructor;
use uuid::Uuid;

// Instance of a pipeline
#[derive(Constructor, Debug)]
pub struct Job {
    pub id: Uuid,
    pub pipeline: Pipeline,
}

// Instance of a stage
pub struct StageRun {
    id: Uuid,
    stage: PStage,
}

// Instance of a step
pub struct StepRun {
    id: Uuid,
    step: PStep,
}
