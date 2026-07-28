use crate::error::ConfigError;
use scylla_core::infrastructure::DatabaseConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CoreConfig {
    #[serde(default)]
    pub grpc: GrpcConfig,

    #[serde(default)]
    pub database: DatabaseConfig,

    #[serde(default)]
    pub cors: CorsConfig,

    #[serde(default)]
    pub bootstrap: Option<BootstrapConfig>,

    #[serde(default)]
    pub metering: MeteringConfig,

    /// SMTP settings. When absent, a no-op mailer is used.
    #[serde(default)]
    pub mail: Option<MailConfig>,

    /// OAuth providers.
    #[serde(default)]
    pub oauth: OauthConfig,

    /// Project-secret encryption. When absent, the secret store is disabled and
    /// secret operations error with a clear message.
    #[serde(default)]
    pub secrets: Option<SecretsConfig>,

    /// Inbound webhook ingress (a separate HTTP listener). When absent, no
    /// webhook server is started and webhook triggers can only be fired manually.
    #[serde(default)]
    pub webhook: Option<WebhookConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebhookConfig {
    /// Address the webhook HTTP server binds to, e.g. `0.0.0.0:8088`.
    pub address: SocketAddr,

    /// Public base URL advertised in `TriggerView.webhook_url`, e.g.
    /// `https://hooks.example.com`. When absent, `webhook_url` is left empty.
    #[serde(default)]
    pub public_base_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretsConfig {
    /// Master key for project-secret AEAD encryption, as 64 hex chars (32 bytes).
    /// Keep it out of source control in real deployments (inject at deploy time
    /// via [`MASTER_KEY_ENV`]).
    pub master_key: String,
}

/// Environment variable that overrides the project-secret master key, so real
/// deployments inject it at deploy time instead of committing it to a config
/// file. Set, it takes precedence over `[secrets].master_key` and enables the
/// secret store even when the file omits `[secrets]`.
pub const MASTER_KEY_ENV: &str = "SCYLLA_MASTER_KEY";

/// The master key committed in the shipped dev/demo config (`config/docker.toml`).
/// It is public, so using it in a real deployment makes every project secret and
/// webhook HMAC secret trivially decryptable by anyone with the repo. Detected at
/// startup to warn loudly.
pub const DEV_MASTER_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OauthConfig {
    /// GitHub OAuth app credentials. When absent, the OAuth service is not
    /// registered.
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
    /// Sender, e.g. `"Scylla <no-reply@scylla.dev>"` or `"no-reply@scylla.dev"`.
    pub from: String,
}

fn default_smtp_port() -> u16 {
    465
}

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

    /// Optional email for the bootstrap admin, enabling email login for it.
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

/// Per-organization quotas, enforced on resource creation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MeteringConfig {
    #[serde(default = "default_max_projects_per_org")]
    pub max_projects_per_org: u64,
}

fn default_max_projects_per_org() -> u64 {
    100
}

impl Default for MeteringConfig {
    fn default() -> Self {
        Self {
            max_projects_per_org: default_max_projects_per_org(),
        }
    }
}

fn default_max_age() -> u64 {
    600
}

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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|source| ConfigError::ReadFile {
            path: path_ref.to_path_buf(),
            source,
        })?;
        Ok(toml::from_str(&content)?)
    }

    /// Overlay environment overrides on top of the file config. Currently only
    /// [`MASTER_KEY_ENV`], so a deployment can inject the master key at deploy
    /// time rather than committing it. Call once, right after loading the file.
    pub fn apply_env_overrides(&mut self) {
        if let Ok(key) = std::env::var(MASTER_KEY_ENV) {
            self.override_master_key(&key);
        }
    }

    /// Set the project-secret master key, enabling the secret store if the file
    /// omitted `[secrets]`. A blank value is ignored (an unset/empty env var must
    /// not wipe a file-provided key). Separated from env reading so it is
    /// unit-testable without touching process environment.
    pub fn override_master_key(&mut self, key: &str) {
        let key = key.trim();
        if !key.is_empty() {
            self.secrets = Some(SecretsConfig {
                master_key: key.to_owned(),
            });
        }
    }

    /// Whether the effective master key is the public dev/demo one. A real
    /// deployment using it has no secret confidentiality at all.
    #[must_use]
    pub fn uses_dev_master_key(&self) -> bool {
        self.secrets
            .as_ref()
            .is_some_and(|s| s.master_key.trim().eq_ignore_ascii_case(DEV_MASTER_KEY))
    }

    pub fn print_example() {
        let config = CoreConfig::default();
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
        let mut config = CoreConfig::default();
        assert!(config.secrets.is_none());
        config.override_master_key("aa11bb22");
        assert_eq!(config.secrets.unwrap().master_key, "aa11bb22");
    }

    #[test]
    fn override_master_key_replaces_a_file_provided_key() {
        let mut config = CoreConfig {
            secrets: Some(SecretsConfig {
                master_key: DEV_MASTER_KEY.to_owned(),
            }),
            ..CoreConfig::default()
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
        // An unset/empty env var must never wipe a file-provided key.
        let mut config = CoreConfig {
            secrets: Some(SecretsConfig {
                master_key: "real-key".to_owned(),
            }),
            ..CoreConfig::default()
        };
        config.override_master_key("   ");
        assert_eq!(config.secrets.unwrap().master_key, "real-key");
    }

    #[test]
    fn uses_dev_master_key_detects_the_public_key() {
        let mut config = CoreConfig::default();
        assert!(!config.uses_dev_master_key(), "no secrets configured");

        config.override_master_key(DEV_MASTER_KEY);
        assert!(config.uses_dev_master_key());
        // Case-insensitive, whitespace-tolerant.
        config.override_master_key(&format!("  {}  ", DEV_MASTER_KEY.to_uppercase()));
        assert!(config.uses_dev_master_key());

        config.override_master_key("a-real-unique-production-key");
        assert!(!config.uses_dev_master_key());
    }
}
