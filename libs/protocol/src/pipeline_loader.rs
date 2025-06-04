use crate::{Pipeline, PipelineStep};
use std::error::Error;
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Trait for loading pipelines from different sources
pub trait PipelineLoader {
    /// Load a pipeline from a source
    fn load_pipeline(&self) -> Result<Pipeline, Box<dyn Error>>;
}

/// Loader for TOML files
pub struct TomlPipelineLoader<P: AsRef<Path>> {
    path: P,
}

impl<P: AsRef<Path>> TomlPipelineLoader<P> {
    /// Create a new TOML pipeline loader
    pub fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P: AsRef<Path>> PipelineLoader for TomlPipelineLoader<P> {
    fn load_pipeline(&self) -> Result<Pipeline, Box<dyn Error>> {
        let toml_str = fs::read_to_string(self.path.as_ref())?;
        let pipeline_toml: PipelineToml = toml::from_str(&toml_str)?;
        Ok(Pipeline {
            id: Uuid::new_v4(),
            name: pipeline_toml.name,
            steps: pipeline_toml.step,
        })
    }
}

/// Loader for JSON files
pub struct JsonPipelineLoader<P: AsRef<Path>> {
    path: P,
}

impl<P: AsRef<Path>> JsonPipelineLoader<P> {
    /// Create a new JSON pipeline loader
    pub fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P: AsRef<Path>> PipelineLoader for JsonPipelineLoader<P> {
    fn load_pipeline(&self) -> Result<Pipeline, Box<dyn Error>> {
        let json_str = fs::read_to_string(self.path.as_ref())?;
        let pipeline: Pipeline = serde_json::from_str(&json_str)?;
        Ok(pipeline)
    }
}

#[derive(serde::Deserialize)]
struct PipelineToml {
    name: String,
    step: Vec<PipelineStep>,
}

// Maintain backward compatibility
pub fn load_pipeline_from_toml(path: &str) -> Result<Pipeline, Box<dyn Error>> {
    let loader = TomlPipelineLoader::new(path);
    loader.load_pipeline()
}
