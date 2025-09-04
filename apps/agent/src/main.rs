//mod command;
#[allow(dead_code)]
mod config;
mod executors;
mod model;

use crate::config::AgentConfig;
use crate::executors::local::LocalExecutor;
use crate::model::executor::PipelineRunner;
use crate::model::pipeline::Pipeline;
use crate::model::shell::Shell;
use crate::model::stage::Stage;
use crate::model::step::Step;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .init();
    // Parse command-line arguments using clap
    let args = Args::parse();

    // Handle the print-example-config flag
    if args.print_example_config {
        AgentConfig::print_example_toml();
        return Ok(());
    }

    // Load configuration from a file if specified, otherwise use default
    let _config = match args.config {
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
            let defaut_config = AgentConfig::default();
            info!(
                "No configuration file specified, using default configuration : {:#?}",
                defaut_config
            );
            defaut_config
        }
    };

    let pipeline = Pipeline {
        name: "demo".to_string(),
        stages: vec![Stage {
            name: "build".to_string(),
            steps: vec![
                Step {
                    name: "echo_1".to_string(),
                    shell: Shell::Sh,
                    command: "echo".to_string(),
                    args: vec!["From".into(), "sh".into(), "$RUST_LOG".into()],
                },
                Step {
                    name: "echo_2".to_string(),
                    shell: Shell::Bash,
                    command: "echo".to_string(),
                    args: vec!["From".into(), "bash".into()],
                },
            ],
        }],
    };

    let executor = LocalExecutor::new();
    let runner = PipelineRunner::new(executor)
        .with_workdir(".")
        .with_env_var("RUST_LOG", "info");

    runner.run_pipeline(&pipeline).await?;
    Ok(())
}
