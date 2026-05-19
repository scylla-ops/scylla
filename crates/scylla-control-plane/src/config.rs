use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use thiserror::Error;

/// Unified control-plane configuration.
///
/// Wraps the existing `scylla_api::CoreConfig` (database / cors / api.grpc /
/// broker-client / bootstrap) and adds the broker SERVER bind address so the
/// in-process broker can listen on a known port. The same TOML file boots the
/// entire control plane.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ControlPlaneConfig {
    #[serde(flatten)]
    pub api: scylla_api::CoreConfig,

    #[serde(default)]
    pub broker_server: BrokerServerConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrokerServerConfig {
    pub addr: SocketAddr,
    #[serde(default = "default_channel_capacity")]
    pub channel_capacity: usize,
}

fn default_channel_capacity() -> usize {
    8192
}

impl Default for BrokerServerConfig {
    fn default() -> Self {
        Self {
            // SAFETY: hardcoded literal; only fails on programmer typo.
            addr: "0.0.0.0:50052".parse().expect("default broker addr"),
            channel_capacity: default_channel_capacity(),
        }
    }
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
        let content = std::fs::read_to_string(path_ref).map_err(|source| ConfigError::ReadFile {
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
}
