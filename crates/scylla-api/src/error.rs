use scylla_core::domain::errors::DomainError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse TOML config: {0}")]
    ParseToml(#[from] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("invalid bootstrap username: {0}")]
    InvalidUsername(#[source] DomainError),
    #[error("invalid bootstrap password: {0}")]
    InvalidPassword(#[source] DomainError),
    #[error("failed to create bootstrap user: {0}")]
    CreateUser(#[source] DomainError),
    #[error("failed to grant admin permissions: {0}")]
    GrantPermission(#[source] DomainError),
}

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("config: {0}")]
    Config(#[from] ConfigError),
    #[error("database initialization: {0}")]
    Database(#[from] DomainError),
    #[error("permission service init: {0}")]
    Permission(String),
    #[error("broker connection to {url}: {message}")]
    BrokerConnect { url: String, message: String },
    #[error("bootstrap: {0}")]
    Bootstrap(#[from] BootstrapError),
    #[error("gRPC reflection: {0}")]
    Reflection(String),
    #[error("gRPC serve: {0}")]
    Serve(#[from] tonic::transport::Error),
}
