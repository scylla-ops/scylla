use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ControlPlaneConfig {
    #[serde(default, alias = "grpc")]
    pub server: ServerConfig,

    #[serde(default)]
    pub ui: UiConfig,

    #[serde(default)]
    pub database: DatabaseConfig,

    #[serde(default)]
    pub cors: CorsConfig,

    #[serde(default)]
    pub bootstrap: Option<BootstrapConfig>,

    #[serde(default)]
    pub mail: Option<MailConfig>,

    #[serde(default)]
    pub oauth: OauthConfig,

    #[serde(default)]
    pub secrets: Option<SecretsConfig>,

    #[serde(default)]
    pub webhook: Option<WebhookConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_acquire_timeout", with = "humantime_serde")]
    pub acquire_timeout: Duration,
    #[serde(default)]
    pub run_migrations: bool,
}

const fn default_max_connections() -> u32 {
    16
}

const fn default_min_connections() -> u32 {
    1
}

const fn default_acquire_timeout() -> Duration {
    Duration::from_secs(30)
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://scylla:scylla@localhost:5432/scylla".to_string(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            acquire_timeout: default_acquire_timeout(),
            run_migrations: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebhookConfig {
    #[serde(default)]
    pub public_base_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretsConfig {
    pub master_key: String,
}

pub const MASTER_KEY_ENV: &str = "SCYLLA_MASTER_KEY";

/// Public (committed in `config/docker.toml`): detected at startup to warn.
pub const DEV_MASTER_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OauthConfig {
    #[serde(default)]
    pub github: Option<GitHubOauthConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubOauthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MailConfig {
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

fn default_smtp_port() -> u16 {
    465
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: SocketAddr,

    #[serde(default)]
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,

    #[serde(default)]
    pub dir: Option<PathBuf>,
}

fn enabled_by_default() -> bool {
    true
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            enabled: enabled_by_default(),
            dir: None,
        }
    }
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

    #[serde(default)]
    pub email: Option<String>,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            username: "admin".to_string(),
            password: "admin123".to_string(),
            email: None,
        }
    }
}

fn default_max_age() -> u64 {
    600
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], 8080)),
            tls: None,
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

impl ControlPlaneConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|source| ConfigError::ReadFile {
            path: path_ref.to_path_buf(),
            source,
        })?;
        Ok(toml::from_str(&content)?)
    }

    pub fn apply_env_overrides(&mut self) {
        if let Ok(key) = std::env::var(MASTER_KEY_ENV) {
            self.override_master_key(&key);
        }
    }

    /// A blank value is ignored: an unset env var must not wipe a file-provided key.
    pub fn override_master_key(&mut self, key: &str) {
        let key = key.trim();
        if !key.is_empty() {
            self.secrets = Some(SecretsConfig {
                master_key: key.to_owned(),
            });
        }
    }

    #[must_use]
    pub fn uses_dev_master_key(&self) -> bool {
        self.secrets
            .as_ref()
            .is_some_and(|s| s.master_key.trim().eq_ignore_ascii_case(DEV_MASTER_KEY))
    }

    pub fn print_example() {
        let config = ControlPlaneConfig::default();
        match toml::to_string_pretty(&config) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("Error generating example config: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_master_key_enables_secret_store_when_absent() {
        let mut config = ControlPlaneConfig::default();
        assert!(config.secrets.is_none());
        config.override_master_key("aa11bb22");
        assert_eq!(config.secrets.unwrap().master_key, "aa11bb22");
    }

    #[test]
    fn override_master_key_replaces_a_file_provided_key() {
        let mut config = ControlPlaneConfig {
            secrets: Some(SecretsConfig {
                master_key: DEV_MASTER_KEY.to_owned(),
            }),
            ..ControlPlaneConfig::default()
        };
        config.override_master_key("  deadbeef  ");
        assert_eq!(
            config.secrets.unwrap().master_key,
            "deadbeef",
            "trimmed + replaced"
        );
    }

    #[test]
    fn override_master_key_ignores_blank() {
        let mut config = ControlPlaneConfig {
            secrets: Some(SecretsConfig {
                master_key: "real-key".to_owned(),
            }),
            ..ControlPlaneConfig::default()
        };
        config.override_master_key("   ");
        assert_eq!(config.secrets.unwrap().master_key, "real-key");
    }

    #[test]
    fn uses_dev_master_key_detects_the_public_key() {
        let mut config = ControlPlaneConfig::default();
        assert!(!config.uses_dev_master_key(), "no secrets configured");

        config.override_master_key(DEV_MASTER_KEY);
        assert!(config.uses_dev_master_key());
        config.override_master_key(&format!("  {}  ", DEV_MASTER_KEY.to_uppercase()));
        assert!(config.uses_dev_master_key());

        config.override_master_key("a-real-unique-production-key");
        assert!(!config.uses_dev_master_key());
    }
}
