use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreConfig {
    #[serde(default)]
    pub grpc: GrpcConfig,

    #[serde(default)]
    pub database: DatabaseConfig,

    #[serde(default)]
    pub cors: CorsConfig,

    #[serde(default)]
    pub bootstrap: Option<BootstrapConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrpcConfig {
    pub address: SocketAddr,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CorsConfig {
    #[serde(default = "default_allow_origins")]
    pub allow_origins: Vec<String>,

    #[serde(default = "default_allow_methods")]
    pub allow_methods: Vec<String>,

    #[serde(default = "default_allow_headers")]
    pub allow_headers: Vec<String>,

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
    vec!["content-type".to_string(), "authorization".to_string()]
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

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            grpc: GrpcConfig::default(),
            database: DatabaseConfig::default(),
            cors: CorsConfig::default(),
            bootstrap: None,
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

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "ws://127.0.0.1:8000".to_string(),
            username: "root".to_string(),
            password: "secret".to_string(),
            namespace: "scylla".to_string(),
            database: "core".to_string(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_origins: default_allow_origins(),
            allow_methods: default_allow_methods(),
            allow_headers: default_allow_headers(),
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
