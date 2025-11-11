mod config;
mod executors;
mod grpc;
mod model;

use crate::config::AgentConfig;
use crate::grpc::Agent;
use clap::Parser;
use std::error::Error;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

/// Scylla Agent - A client for the Scylla CI/CD system
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Print an example configuration file and exit
    #[arg(short = 'e', long = "print-example-config")]
    print_example_config: bool,

    /// Path to the configuration file
    #[arg(short, long)]
    config: Option<String>,
}

fn init_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug,h2=warn")),
        )
        .pretty()
        .with_target(true)
        .with_line_number(false)
        .with_file(false)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    // Parse command-line arguments using clap
    let args = Args::parse();

    // Handle the print-example-config flag
    if args.print_example_config {
        AgentConfig::print_example_toml();
        return Ok(());
    }

    // Load configuration from a file if specified, otherwise use default
    let config = match args.config {
        Some(config_path) => match AgentConfig::from_toml_file(&config_path) {
            Ok(config) => {
                info!("Loaded configuration from {}", config_path);
                config
            }
            Err(e) => {
                error!("Failed to load configuration from {}: {}", config_path, e);
                info!("Falling back to default configuration");
                AgentConfig::default()
            }
        },
        None => {
            let default_config = AgentConfig::default();
            info!(
                "No configuration file specified, using default configuration : {:#?}",
                default_config
            );
            default_config
        }
    };

    Agent::new(config.grpc_url).run().await?;

    Ok(())
}
