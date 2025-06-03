use crate::constants::{DEFAULT_CHANNEL_SIZE, DEFAULT_WEBSOCKET_URL};
use crate::error::Result;
use protocol::toml;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RootConfig {
    pub agent_config: AgentConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    pub websocket_url: String,
    pub channel_size: usize,
}

impl AgentConfig {
    pub fn builder() -> AgentConfigBuilder {
        AgentConfigBuilder::default()
    }

    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let root_config: RootConfig =
            toml::from_str(&content).map_err(|e| anyhow::anyhow!("{}", e))?;
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
                println!("{}", toml_str);
            },
            Err(e) => {
                eprintln!("Error generating example TOML: {}", e);
            }
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            websocket_url: DEFAULT_WEBSOCKET_URL.to_string(),
            channel_size: DEFAULT_CHANNEL_SIZE,
        }
    }
}

pub struct AgentConfigBuilder {
    websocket_url: Option<String>,
    channel_size: Option<usize>,
}

impl Default for AgentConfigBuilder {
    fn default() -> Self {
        Self {
            websocket_url: Some(DEFAULT_WEBSOCKET_URL.to_string()),
            channel_size: Some(DEFAULT_CHANNEL_SIZE),
        }
    }
}

impl AgentConfigBuilder {
    pub fn websocket_url<S: Into<String>>(mut self, url: S) -> Self {
        self.websocket_url = Some(url.into());
        self
    }

    pub fn channel_size(mut self, size: usize) -> Self {
        self.channel_size = Some(size);
        self
    }

    pub fn build(self) -> Result<AgentConfig> {
        let websocket_url = self.websocket_url.ok_or_else(|| anyhow::anyhow!("WebSocket URL is required"))?;
        let channel_size = self.channel_size.unwrap_or(DEFAULT_CHANNEL_SIZE);

        Ok(AgentConfig {
            websocket_url,
            channel_size,
        })
    }
}
