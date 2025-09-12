use anyhow::{Context, Result};
use derive_builder::Builder;
use protocol::serde::{Deserialize, Serialize};
use protocol::toml;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

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
#[derive(Debug, Serialize, Deserialize, Clone, Builder)]
pub struct CoreConfig {
    /// Database connection configuration
    pub database_config: DatabaseConfig,
    /// GRPC Api configuration
    pub grpc_config: SocketAddr,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            database_config: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "postgres".to_string(),
                database: "scylla".to_string(),
            },
            grpc_config: SocketAddr::from(([127, 0, 0, 1], 50051)),
        }
    }
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

impl CoreConfig {
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
            .unwrap_or_else(|_| CoreConfig::default());

        let root_config = RootConfig {
            core_config: default_config,
        };

        match toml::to_string_pretty(&root_config) {
            Ok(toml_str) => {
                println!("# Example Core Configuration");
                println!("{toml_str}");
            }
            Err(e) => {
                eprintln!("Error generating example TOML: {e}");
            }
        }
    }
}
