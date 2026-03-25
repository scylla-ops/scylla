use anyhow::{Context, Result};
use scylla_core::infrastructure::DatabaseConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CoreConfig {
    #[cfg(feature = "grpc")]
    #[serde(default)]
    pub grpc: GrpcConfig,

    #[cfg(feature = "rest")]
    #[serde(default)]
    pub rest: RestConfig,

    #[serde(default)]
    pub database: DatabaseConfig,

    #[serde(default)]
    pub cors: CorsConfig,

    #[serde(default)]
    pub bootstrap: Option<BootstrapConfig>,
}

#[cfg(feature = "grpc")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrpcConfig {
    pub address: SocketAddr,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CorsConfig {
    #[serde(default = "default_allow_origins")]
    pub allow_origins: Vec<String>,

    #[serde(default = "default_allow_methods")]
    pub allow_methods: Vec<String>,

    #[serde(default = "default_allow_headers")]
    pub allow_headers: Vec<String>,

    #[serde(default = "default_expose_headers")]
    pub expose_headers: Vec<String>,

    #[serde(default = "default_max_age")]
    pub max_age_seconds: u64,
}

fn default_allow_origins() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_allow_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
        "OPTIONS".to_string(),
    ]
}

fn default_allow_headers() -> Vec<String> {
    vec![
        "content-type".to_string(),
        "authorization".to_string(),
        "x-grpc-web".to_string(),
        "x-user-agent".to_string(),
    ]
}

fn default_expose_headers() -> Vec<String> {
    vec![
        "grpc-status".to_string(),
        "grpc-message".to_string(),
        "grpc-status-details-bin".to_string(),
    ]
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BootstrapConfig {
    pub username: String,

    pub password: String,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        }
    }
}

fn default_max_age() -> u64 {
    600
}

#[cfg(feature = "rest")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RestConfig {
    pub address: SocketAddr,
}

#[cfg(feature = "rest")]
impl Default for RestConfig {
    fn default() -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], 3000)),
        }
    }
}

#[cfg(feature = "grpc")]
impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], 50051)),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_origins: default_allow_origins(),
            allow_methods: default_allow_methods(),
            allow_headers: default_allow_headers(),
            expose_headers: default_expose_headers(),
            max_age_seconds: default_max_age(),
        }
    }
}

impl CoreConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        let config: CoreConfig =
            toml::from_str(&content).with_context(|| "Failed to parse TOML config")?;
        Ok(config)
    }

    pub fn print_example() {
        let config = CoreConfig::default();
        match toml::to_string_pretty(&config) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("Error generating example config: {e}"),
        }
    }
}
