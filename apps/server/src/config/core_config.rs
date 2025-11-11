use anyhow::{Context, Result};
use derive_builder::Builder;
use protocol::serde::{Deserialize, Serialize};
use protocol::toml;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

/// Core configuration for the Scylla core
///
/// Contains all configuration parameters needed to run the core,
/// including network address, database connection, and authentication details.
#[derive(Debug, Serialize, Deserialize, Clone, Builder)]
pub struct CoreConfig {
    /// Database connection configuration
    #[serde(default)]
    pub database_config: DatabaseConfig,

    /// gRPC API configuration
    #[serde(default)]
    pub grpc_config: GrpcConfig,

    /// Authentication configuration
    #[serde(default)]
    pub auth_config: AuthConfig,

    /// RBAC configuration
    #[serde(default)]
    pub rbac_config: RbacConfig,

    /// Bootstrap configuration
    #[serde(default)]
    pub bootstrap_config: BootstrapConfig,

    /// CORS configuration
    #[serde(default)]
    pub cors_config: CorsConfig,
}

/// Database connection configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// Database server URL
    pub url: String,
    /// Database username for authentication
    pub username: String,
    /// Database password for authentication
    pub password: String,
    /// Database namespace to connect to
    pub namespace: String,
    /// Database name to connect to
    pub database: String,
}

/// gRPC API configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrpcConfig {
    /// gRPC server bind address
    pub address: SocketAddr,
}

/// Authentication configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
    /// Token expiration duration in seconds
    pub token_duration_seconds: u64,
    /// Encryption key for tokens (base64 encoded)
    /// If not provided, will be auto-generated (not recommended for production)
    pub token_key: Option<String>,
}

/// RBAC configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RbacConfig {
    /// Path to the Casbin model configuration file
    pub model_path: String,
}

/// Bootstrap configuration for creating the first admin user
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BootstrapConfig {
    /// Enable automatic bootstrap of first admin user
    pub enabled: bool,
    /// Username for the bootstrap admin user
    pub username: String,
    /// Password for the bootstrap admin user
    /// IMPORTANT: Change this password immediately after first login!
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CorsConfig {
    /// Preset configuration: none | permissive | very_permissive
    #[serde(default)]
    pub preset: CorsPreset,

    /// List of allowed origins (as HeaderValue parseable strings)
    #[serde(default)]
    pub allow_origins: Vec<String>,

    /// List of allowed methods (GET, POST, ...)
    #[serde(default)]
    pub allow_methods: Vec<String>,

    /// List of allowed request headers
    #[serde(default)]
    pub allow_headers: Vec<String>,

    /// List of exposed response headers
    #[serde(default)]
    pub expose_headers: Vec<String>,

    /// Allow credentials
    #[serde(default)]
    pub allow_credentials: Option<bool>,

    /// Allow Private Network
    #[serde(default)]
    pub allow_private_network: Option<bool>,

    /// Max age in seconds for preflight caching
    #[serde(default)]
    pub max_age_seconds: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorsPreset {
    None,
    Permissive,
    VeryPermissive,
}

impl Default for CorsPreset {
    fn default() -> Self {
        CorsPreset::Permissive
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            preset: CorsPreset::Permissive,
            allow_origins: Vec::new(),
            allow_methods: Vec::new(),
            allow_headers: Vec::new(),
            expose_headers: Vec::new(),
            allow_credentials: Some(true),
            allow_private_network: None,
            max_age_seconds: Some(600),
        }
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            database_config: DatabaseConfig::default(),
            grpc_config: GrpcConfig::default(),
            auth_config: AuthConfig::default(),
            rbac_config: RbacConfig::default(),
            bootstrap_config: BootstrapConfig::default(),
            cors_config: CorsConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "ws://127.0.0.1:8000".to_string(),
            username: "root".to_string(),
            password: "secret".to_string(),
            namespace: "app".to_string(),
            database: "main".to_string(),
        }
    }
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], 50051)),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token_duration_seconds: 86400, // 24 hours
            token_key: None,
        }
    }
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            model_path: "casbin/model.conf".to_string(),
        }
    }
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            username: "admin".to_string(),
            password: "changeme".to_string(),
        }
    }
}

/// Root configuration container for TOML deserialization
///
/// This struct wraps the core configuration to match the structure
/// of the TOML configuration file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RootConfig {
    pub core_config: CoreConfig,
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

    /// Generates an example TOML configuration based on default values and prints it
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
