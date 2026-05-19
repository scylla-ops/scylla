mod bootstrap;
mod config;
mod error;
mod startup;

use anyhow::{Context, Result};
use clap::Parser;
use config::CoreConfig;

#[derive(Parser, Debug)]
#[command(author, version, about = "Scylla API server")]
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
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scylla_api=info,scylla_core=info,warn".into()),
        )
        .init();

    let args = Args::parse();

    if args.print_example_config {
        CoreConfig::print_example();
        return;
    }

    if let Err(e) = run(args).await {
        tracing::error!("Application error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<()> {
    let config = load_config(&args)?;
    tracing::debug!("Configuration: {:#?}", config);

    let services = startup::init_services(&config).await?;

    startup::start_grpc(&config, &services).await?;

    Ok(())
}

fn load_config(args: &Args) -> Result<CoreConfig> {
    if let Some(config_path) = &args.config {
        CoreConfig::from_file(config_path)
            .with_context(|| format!("Failed to load configuration from {}", config_path))
    } else {
        tracing::info!("No configuration file provided, using defaults");
        Ok(CoreConfig::default())
    }
}
