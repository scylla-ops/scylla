use anyhow::Context;
use anyhow::Result;
use derive_builder::Builder;
use protocol::toml;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use tracing::warn;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RootConfig {
    pub agent_config: AgentConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Builder)]
pub struct AgentConfig {
    pub grpc_url: SocketAddr,
}

impl AgentConfig {
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let root_config: RootConfig =
            toml::from_str(&content).context("Failed to parse TOML config")?;
        Ok(root_config.agent_config)
    }

    /// Generates an example TOML configuration based on the default values
    /// and prints it to the terminal.
    pub fn print_example_toml() {
        let default_config = Self::default();
        let root_config = RootConfig {
            agent_config: default_config,
        };
        match toml::to_string_pretty(&root_config) {
            Ok(toml_str) => {
                println!("# Example Agent Configuration");
                println!("{toml_str}");
            }
            Err(e) => {
                warn!("Error generating example TOML: {e}");
            }
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            grpc_url: SocketAddr::from(([127, 0, 0, 1], 50051)),
        }
    }
}
