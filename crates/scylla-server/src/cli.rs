//! The command line and process setup every edition binary shares.
//!
//! An edition's `main.rs` is the sequence: parse, initialise tracing, load the
//! configuration, open the pool, build a [`crate::Server`] and serve it.
//! Everything on that path that does not depend on the edition lives here.

use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches, Parser};
use scylla_core::config::{ControlPlaneConfig, MASTER_KEY_ENV};
use std::path::Path;

/// The options of a control-plane binary.
#[derive(Parser, Debug)]
pub struct Cli {
    /// Print an example configuration file and exit
    #[arg(short = 'e', long = "print-example-config")]
    pub print_example_config: bool,

    /// Path to the configuration file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Serve the API only, without the web UI. The dev loop this exists for is
    /// `pnpm dev`: Vite owns the UI on :5173 and this binary has no `dist/` to
    /// serve, so the SPA fallback would answer every unmatched path with a
    /// "no UI bundled" page. Overrides `[ui].enabled`.
    #[arg(long = "no-ui")]
    pub no_ui: bool,
}

impl Cli {
    /// Parse the process arguments under the binary's own identity. Left to
    /// clap's defaults, `--help` and `--version` would print this library's
    /// name and version instead of the edition's: that is what a bare
    /// `<Cli as clap::Parser>::parse()` does, so do not call it.
    #[must_use]
    pub fn parse_as(name: &'static str, version: &'static str, about: &'static str) -> Self {
        let matches = Self::command()
            .name(name)
            .version(version)
            .about(about)
            .get_matches();
        Self::from_arg_matches(&matches).unwrap_or_else(|e| e.exit())
    }

    /// The configuration file (or the defaults when none was given) with the
    /// environment overrides and the command-line flags applied. Logs which
    /// optional subsystems are configured, and refuses to stay quiet about the
    /// public dev master key.
    pub fn load_config(&self) -> Result<ControlPlaneConfig> {
        let mut config = if let Some(path) = &self.config {
            ControlPlaneConfig::from_file(Path::new(path))
                .with_context(|| format!("Failed to load configuration from {path}"))?
        } else {
            tracing::info!("No configuration file provided, using defaults");
            ControlPlaneConfig::default()
        };

        // Let a deployment inject the project-secret master key at deploy time
        // instead of committing it to a config file.
        config.apply_env_overrides();

        // The flag only ever turns the UI off, never on: `--no-ui` is a dev
        // convenience, not a way to resurrect assets that were never built in.
        if self.no_ui {
            config.ui.enabled = false;
        }

        // The shipped dev/demo config carries a PUBLIC master key. Using it in a
        // real deployment leaves every project secret and webhook secret
        // decryptable by anyone with the repo, so refuse to stay quiet about it.
        if config.uses_dev_master_key() {
            tracing::error!(
                "SECURITY: project secrets are encrypted with the PUBLIC dev master key, so they \
                 are NOT confidential. Set {MASTER_KEY_ENV} to a unique 64-hex-char key before \
                 exposing this instance."
            );
        }

        // Never debug-dump the whole config: it holds the master key, the
        // bootstrap password, SMTP and OAuth secrets. Log only which optional
        // subsystems are configured.
        tracing::info!(
            ui = config.ui.enabled,
            secrets = config.secrets.is_some(),
            mail = config.mail.is_some(),
            github_oauth = config.oauth.github.is_some(),
            webhook = config.webhook.is_some(),
            "configuration loaded",
        );

        Ok(config)
    }
}

/// Install the process-wide tracing subscriber.
///
/// `RUST_LOG` wins when set. Otherwise: `audit=warn` shows denials and hides
/// grants. Every authorization decision is also written to the `audit_log`
/// table, which stays complete either way, so the log copy of a *granted*
/// action is duplicate detail that drowns everything else (it accounted for 244
/// of 303 lines on a ten-minute run with almost no traffic). Denials stay
/// visible because they are what someone reads logs for. `RUST_LOG=audit=info`
/// brings the full trail back with no rebuild.
///
/// The core crates log at `info`; `edition_targets` are the edition's own
/// crate names, logged at the same level.
pub fn init_tracing(edition_targets: &[&str]) {
    let core_targets = [
        "scylla_server",
        "scylla_core",
        "scylla_auth",
        "scylla_db",
        "scylla_domain",
    ];
    let at_info = edition_targets
        .iter()
        .chain(core_targets.iter())
        .map(|target| format!("{target}=info"))
        .collect::<Vec<_>>()
        .join(",");
    let filter = format!("audit=warn,{at_info},warn");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .init();
}
