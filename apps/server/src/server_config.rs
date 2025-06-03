use protocol::serde::{Deserialize, Serialize};
use protocol::toml;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use anyhow::{Context, Result, anyhow};

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 3000;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RootConfig {
    pub server_config: CoreConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreConfig {
    pub addr: SocketAddr,
}

impl CoreConfig {
    pub fn builder() -> CoreConfigBuilder {
        CoreConfigBuilder::default()
    }

    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        let root_config: RootConfig = toml::from_str(&content)
            .with_context(|| "Failed to parse TOML config")?;
        Ok(root_config.server_config)
    }

    /// Generates an example TOML configuration based on the default values
    /// and prints it to the terminal.
    pub fn print_example_toml() {
        let default_config = CoreConfigBuilder::default().build().unwrap_or_else(|_| {
            CoreConfig {
                addr: format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT).parse().unwrap(),
            }
        });

        let root_config = RootConfig {
            server_config: default_config,
        };

        match toml::to_string_pretty(&root_config) {
            Ok(toml_str) => {
                println!("# Example Server Configuration");
                println!("{}", toml_str);
            },
            Err(e) => {
                eprintln!("Error generating example TOML: {}", e);
            }
        }
    }
}

pub struct CoreConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
}

impl Default for CoreConfigBuilder {
    fn default() -> Self {
        Self {
            host: Some(DEFAULT_HOST.to_string()),
            port: Some(DEFAULT_PORT),
        }
    }
}

impl CoreConfigBuilder {
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn build(self) -> Result<CoreConfig> {
        let host = self.host.ok_or_else(|| anyhow!("Host is required"))?;
        let port = self.port.ok_or_else(|| anyhow!("Port is required"))?;
        let socket_addr = format!("{}:{}", host, port)
            .parse()
            .with_context(|| format!("Invalid IP address or port: {}:{}", host, port))?;

        Ok(CoreConfig {
            addr: socket_addr,
        })
    }
}
