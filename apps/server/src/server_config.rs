use protocol::serde::{Deserialize, Serialize};
use protocol::toml;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 3000;
pub const DEFAULT_MAX_BUFFER_SIZE: usize = 4096;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreConfig {
    pub addr: SocketAddr,
    pub max_buffer_size: usize,
}

impl CoreConfig {
    pub fn builder() -> CoreConfigBuilder {
        CoreConfigBuilder::default()
    }

    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}

pub struct CoreConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_buffer_size: Option<usize>,
}

impl Default for CoreConfigBuilder {
    fn default() -> Self {
        Self {
            host: Some(DEFAULT_HOST.to_string()),
            port: Some(DEFAULT_PORT),
            max_buffer_size: Some(DEFAULT_MAX_BUFFER_SIZE),
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

    pub fn max_buffer_size(mut self, max_buffer_size: usize) -> Self {
        self.max_buffer_size = Some(max_buffer_size);
        self
    }

    pub fn build(self) -> Result<CoreConfig, &'static str> {
        let host = self.host.ok_or("Host est requis")?;
        let port = self.port.ok_or("Port est requis")?;
        let socket_addr = format!("{}:{}", host, port)
            .parse()
            .map_err(|_| "Adresse IP ou port invalide")?;

        let max_buffer_size = self.max_buffer_size.unwrap_or(DEFAULT_MAX_BUFFER_SIZE);

        Ok(CoreConfig {
            addr: socket_addr,
            max_buffer_size,
        })
    }
}
