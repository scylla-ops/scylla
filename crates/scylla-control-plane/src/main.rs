use anyhow::{Context, Result};
use clap::Parser;
use scylla_control_plane::config::{ControlPlaneConfig, MASTER_KEY_ENV};

#[derive(Parser, Debug)]
#[command(author, version, about = "Scylla control-plane (gRPC API)")]
struct Args {
    /// Print an example configuration file and exit
    #[arg(short = 'e', long = "print-example-config")]
    print_example_config: bool,

    /// Path to the configuration file
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "audit=info,scylla_control_plane=info,scylla_core=info,warn".into()
            }),
        )
        .init();

    let args = Args::parse();

    if args.print_example_config {
        ControlPlaneConfig::print_example();
        return;
    }

    if let Err(e) = run(args).await {
        tracing::error!("Application error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<()> {
    let config = load_config(&args)?;
    // Never debug-dump the whole config: it holds the master key, the bootstrap
    // password, SMTP and OAuth secrets. Log only which optional subsystems are
    // configured.
    tracing::info!(
        secrets = config.secrets.is_some(),
        mail = config.mail.is_some(),
        github_oauth = config.oauth.github.is_some(),
        webhook = config.webhook.is_some(),
        "configuration loaded",
    );
    scylla_control_plane::runtime::run(config).await
}

fn load_config(args: &Args) -> Result<ControlPlaneConfig> {
    let mut config = if let Some(config_path) = &args.config {
        ControlPlaneConfig::from_file(config_path)
            .with_context(|| format!("Failed to load configuration from {}", config_path))?
    } else {
        tracing::info!("No configuration file provided, using defaults");
        ControlPlaneConfig::default()
    };

    // Let a deployment inject the project-secret master key at deploy time
    // instead of committing it to a config file.
    config.apply_env_overrides();

    // The shipped dev/demo config carries a PUBLIC master key. Using it in a real
    // deployment leaves every project secret and webhook secret decryptable by
    // anyone with the repo, so refuse to stay quiet about it.
    if config.uses_dev_master_key() {
        tracing::error!(
            "SECURITY: project secrets are encrypted with the PUBLIC dev master key, so they are \
             NOT confidential. Set {} to a unique 64-hex-char key before exposing this instance.",
            MASTER_KEY_ENV,
        );
    }

    Ok(config)
}
