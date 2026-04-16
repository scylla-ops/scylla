use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "scylla-agent", about = "Scylla pipeline execution agent")]
pub struct AgentConfig {
    /// Hermes broker gRPC URL
    #[arg(long, default_value = "http://127.0.0.1:50052")]
    pub broker_url: String,

    /// Queue group name for load-balanced dispatch
    #[arg(long, default_value = "agents")]
    pub queue_group: String,

    /// Subject to subscribe to for job dispatch
    #[arg(long, default_value = "scylla.jobs.dispatch")]
    pub dispatch_subject: String,

    /// Persistent agent identifier. If absent, a fresh ulid is generated
    /// (NOT recommended for production; set via env or flag to keep identity stable).
    #[arg(long, env = "SCYLLA_AGENT_ID")]
    pub agent_id: Option<String>,

    /// Hostname reported to the registry. Defaults to the OS hostname.
    #[arg(long, env = "SCYLLA_AGENT_HOSTNAME")]
    pub hostname: Option<String>,

    /// Heartbeat publish interval in seconds (must be >= 1).
    #[arg(long, default_value_t = 5, value_parser = clap::value_parser!(u64).range(1..))]
    pub heartbeat_interval_secs: u64,
}

impl AgentConfig {
    /// Returns the configured agent id, generating a ulid as last-resort fallback.
    #[must_use]
    pub fn resolved_agent_id(&self) -> String {
        self.agent_id
            .clone()
            .unwrap_or_else(|| ulid::Ulid::new().to_string().to_lowercase())
    }

    /// Returns the configured hostname, falling back to the OS hostname or "unknown".
    #[must_use]
    pub fn resolved_hostname(&self) -> String {
        self.hostname.clone().unwrap_or_else(|| {
            hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string())
        })
    }
}
