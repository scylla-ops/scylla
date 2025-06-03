use crate::{Pipeline, PipelineStep};
use std::fs;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct PipelineToml {
    name: String,
    step: Vec<PipelineStep>,
}

pub fn load_pipeline_from_toml(path: &str) -> Result<Pipeline, Box<dyn std::error::Error>> {
    let toml_str = fs::read_to_string(path)?;
    let pipeline_toml: PipelineToml = toml::from_str(&toml_str)?;
    Ok(Pipeline {
        id: Uuid::new_v4(),
        name: pipeline_toml.name,
        steps: pipeline_toml.step,
    })
}
