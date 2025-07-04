mod agents;
mod api;
mod config;
mod core;
mod database;

// Internal crate imports
use crate::api::ApiBuilder;
use crate::config::{CoreConfig, MAX_CHANNEL_SIZE};
use crate::database::{DieselDatabase, SqlxDatabase};

// External crate imports
use anyhow::{Result, anyhow};
use clap::Parser;
use protocol::Message;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

/// Scylla Core - The core component for the Scylla CI/CD system
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

pub struct AppState {
    pub core_tx: mpsc::Sender<Message>,
}

fn load_config(args: &Args) -> Result<CoreConfig> {
    if let Some(config_path) = &args.config {
        match CoreConfig::from_toml_file(config_path) {
            Ok(config) => Ok(config),
            Err(e) => {
                error!("Failed to load configuration from {}: {}", config_path, e);
                info!("Falling back to default configuration");
                CoreConfig::try_create_default_config()
            }
        }
    } else {
        CoreConfig::try_create_default_config()
    }
}

async fn init_database_pools(core_config: &CoreConfig) -> Result<(DieselDatabase, SqlxDatabase)> {
    // Create database config from core config
    let db_config = database::DatabaseConfig::new(
        core_config.database.host.clone(),
        core_config.database.port,
        core_config.database.username.clone(),
        core_config.database.password.clone(),
        core_config.database.database.clone(),
    );

    // Initialize database and run migrations
    let diesel_db = DieselDatabase::new(&db_config)
        .map_err(|e| anyhow!("Database connection failed: {}", e))?;

    diesel_db
        .run_migrations()
        .map_err(|e| anyhow!("Database migration failed: {}", e))?;

    // Initialize SQLx pool for session store
    let sqlx_db = SqlxDatabase::new(&db_config)
        .await
        .map_err(|e| anyhow!("SQLx database connection failed: {}", e))?;

    info!("Database connection established and migrations completed successfully");
    Ok((diesel_db, sqlx_db))
}

async fn spawn_core(
    core_rx: mpsc::Receiver<Message>,
    core_config: CoreConfig,
) -> tokio::task::JoinHandle<()> {
    let mut core = core::Core::new(core_config, core_rx);

    tokio::spawn(async move {
        if let Err(e) = core.run().await {
            error!("{e:#}");
        }
    })
}

async fn run_api(
    core_config: &CoreConfig,
    (diesel_database, sqlx_database): (DieselDatabase, SqlxDatabase),
    app_state: Arc<AppState>,
) -> Result<()> {
    let listener = TcpListener::bind(core_config.addr).await?;

    info!("Api running on {}", core_config.addr);

    let api_builder = ApiBuilder::new((&diesel_database, &sqlx_database));
    let app = api_builder.build_v1_api(app_state).await;

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow!("API error: {}", e))
}

async fn start_application(core_config: CoreConfig) -> Result<()> {
    let (core_tx, core_rx) = mpsc::channel::<Message>(MAX_CHANNEL_SIZE);

    // Create app state for API handlers
    let app_state = Arc::new(AppState {
        core_tx: core_tx.clone(),
    });

    // Start core in background
    let core_task = spawn_core(core_rx, core_config.clone()).await;

    // Initialize databases with migrations
    let (diesel_db, sqlx_db) = init_database_pools(&core_config).await?;
    // Run the API
    let api_task = run_api(&core_config, (diesel_db, sqlx_db), app_state);

    // Wait for either task to complete
    tokio::select! {
        result = api_task => {
            if let Err(e) = result {
                error!("API error: {}", e);
            }
        }
        _ = core_task => {
            info!("Core processing task completed");
        }
    }

    Ok(())
}

fn init_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    let args = Args::parse();

    if args.print_example_config {
        CoreConfig::print_example_toml();
        return Ok(());
    }

    info!("Core starting");
    let config = load_config(&args)?;
    start_application(config).await
}
