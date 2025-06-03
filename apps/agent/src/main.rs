mod command;
#[allow(dead_code)]
mod config;
mod constants;
#[allow(dead_code)]
mod error;
mod job;
mod state;
mod websocket;

use crate::config::AgentConfig;
use crate::job::JobExecutor;
use crate::state::new_shared_state;
use crate::websocket::WebSocketClient;
use clap::Parser;
use std::error::Error;
use std::sync::Arc;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    // Parse command-line arguments using clap
    let args = Args::parse();

    // Handle the print-example-config flag
    if args.print_example_config {
        AgentConfig::print_example_toml();
        return Ok(());
    }

    let state = new_shared_state();

    let job_executor = Arc::new(JobExecutor::new(None, state.clone()));

    // Load configuration from file if specified, otherwise use default
    let config = match args.config {
        Some(config_path) => {
            match AgentConfig::from_toml_file(&config_path) {
                Ok(config) => {
                    tracing::info!("Loaded configuration from {}", config_path);
                    config
                },
                Err(e) => {
                    tracing::error!("Failed to load configuration from {}: {}", config_path, e);
                    tracing::info!("Falling back to default configuration");
                    AgentConfig::default()
                }
            }
        },
        None => AgentConfig::default(),
    };

    loop {
        let client = WebSocketClient::builder()
            .config(config.clone())
            .state(state.clone())
            .job_executor(job_executor.clone())
            .build()?;

        match client.run().await {
            Ok(_) => {
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket connection lost: {}", e);
                tracing::info!("Reconnecting in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }

    Ok(())
}
