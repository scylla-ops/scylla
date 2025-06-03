use anyhow::{Context, Result, anyhow};
use protocol::serde::{Deserialize, Serialize};
use protocol::toml;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 3000;

/// Root configuration container for TOML deserialization
///
/// This struct wraps the core configuration to match the structure
/// of the TOML configuration file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RootConfig {
    pub core_config: CoreConfig,
}

/// Core configuration for the Scylla core
///
/// Contains all configuration parameters needed to run the core,
/// including network address and database connection details.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreConfig {
    /// Socket address for the core to listen on
    pub addr: SocketAddr,
    /// Database connection configuration
    #[serde(default = "default_database_config")]
    pub database: DatabaseConfig,
}

/// Database connection configuration
///
/// Contains all parameters needed to establish a connection to the database.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// Database server hostname or IP address
    pub host: String,
    /// Database server port
    pub port: u16,
    /// Database username for authentication
    pub username: String,
    /// Database password for authentication
    pub password: String,
    /// Database name to connect to
    pub database: String,
}

/// Creates a default database configuration with standard PostgreSQL settings
///
/// # Returns
/// * `DatabaseConfig` - Default database configuration
fn default_database_config() -> DatabaseConfig {
    DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "postgres".to_string(),
        password: "postgres".to_string(),
        database: "scylla".to_string(),
    }
}

impl CoreConfig {
    /// Creates a new configuration builder with default values
    ///
    /// # Returns
    /// * `CoreConfigBuilder` - Builder for creating a CoreConfig
    pub fn builder() -> CoreConfigBuilder {
        CoreConfigBuilder::default()
    }

    /// Loads configuration from a TOML file
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    /// * `Result<Self>` - Loaded configuration or error
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        let root_config: RootConfig =
            toml::from_str(&content).with_context(|| "Failed to parse TOML config")?;
        Ok(root_config.core_config)
    }

    /// Generates an example TOML configuration based on the default values
    /// and prints it to the terminal.
    pub fn print_example_toml() {
        let default_config = CoreConfigBuilder::default()
            .build()
            .unwrap_or_else(|_| CoreConfig {
                addr: format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT)
                    .parse()
                    .unwrap(),
                database: default_database_config(),
            });

        let root_config = RootConfig {
            core_config: default_config,
        };

        match toml::to_string_pretty(&root_config) {
            Ok(toml_str) => {
                println!("# Example Core Configuration");
                println!("{}", toml_str);
            }
            Err(e) => {
                eprintln!("Error generating example TOML: {}", e);
            }
        }
    }
}

/// Builder for creating CoreConfig instances
///
/// Provides a fluent interface for setting configuration parameters
/// with reasonable defaults.
pub struct CoreConfigBuilder {
    /// Core hostname or IP address
    host: Option<String>,
    /// Core port number
    port: Option<u16>,
}

impl Default for CoreConfigBuilder {
    /// Creates a new builder with default values
    ///
    /// # Returns
    /// * `Self` - Builder with default values
    fn default() -> Self {
        Self {
            host: Some(DEFAULT_HOST.to_string()),
            port: Some(DEFAULT_PORT),
        }
    }
}

impl CoreConfigBuilder {
    /// Sets the host for the core
    ///
    /// # Arguments
    /// * `host` - Hostname or IP address
    ///
    /// # Returns
    /// * `Self` - Builder with updated host
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets the port for the core
    ///
    /// # Arguments
    /// * `port` - Port number
    ///
    /// # Returns
    /// * `Self` - Builder with updated port
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Builds a CoreConfig from the builder
    ///
    /// # Returns
    /// * `Result<CoreConfig>` - Built configuration or error
    pub fn build(self) -> Result<CoreConfig> {
        let host = self.host.ok_or_else(|| anyhow!("Host is required"))?;
        let port = self.port.ok_or_else(|| anyhow!("Port is required"))?;
        let socket_addr = format!("{}:{}", host, port)
            .parse()
            .with_context(|| format!("Invalid IP address or port: {}:{}", host, port))?;

        Ok(CoreConfig {
            addr: socket_addr,
            database: default_database_config(),
        })
    }
}
