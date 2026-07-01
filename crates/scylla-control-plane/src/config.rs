use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Unified control-plane configuration. Wraps [`scylla_api::CoreConfig`]
/// (database / cors / gRPC / bootstrap); the same TOML file boots the whole
/// in-process control plane. Job dispatch is in-process, so there is no broker
/// configuration.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ControlPlaneConfig {
    #[serde(flatten)]
    pub api: scylla_api::CoreConfig,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    ReadFile {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse TOML config: {0}")]
    ParseToml(#[from] toml::de::Error),
}

impl ControlPlaneConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        let content =
            std::fs::read_to_string(path_ref).map_err(|source| ConfigError::ReadFile {
                path: path_ref.to_path_buf(),
                source,
            })?;
        Ok(toml::from_str(&content)?)
    }

    pub fn print_example() {
        let config = Self::default();
        match toml::to_string_pretty(&config) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("Error generating example config: {e}"),
        }
    }

    /// Overlay environment overrides on the loaded config (see
    /// [`scylla_api::CoreConfig::apply_env_overrides`]).
    pub fn apply_env_overrides(&mut self) {
        self.api.apply_env_overrides();
    }

    /// Whether the effective project-secret master key is the public dev one.
    #[must_use]
    pub fn uses_dev_master_key(&self) -> bool {
        self.api.uses_dev_master_key()
    }
}
