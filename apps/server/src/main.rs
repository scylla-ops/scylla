mod agents;
mod api;
mod config;
mod core;
mod database;
mod tcp_server;

use std::net::SocketAddr;
// Internal crate imports
use crate::api::ApiBuilder;
use crate::config::{CoreConfig, MAX_CHANNEL_SIZE};
use crate::database::{DieselDatabase, SqlxDatabase};

// External crate imports
use crate::api::grpc::{AuthService, UserRepositoryDiesel, UserService};
use anyhow::{Result, anyhow};
use clap::Parser;
use pasetors::keys::{Generate, SymmetricKey};
use protocol::services::auth_service_server::AuthServiceServer;
use protocol::services::user_service_server::UserServiceServer;
use protocol::tonic::transport::Server;
use protocol::{Message, services};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tracing::info;
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
                panic!("Failed to load configuration from {config_path}: {e}");
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

    // Start core
    //let core_task = Core::spawn_core(core_rx, core_config.clone());

    // Start TCP server for agents
    //let tcp_task = TcpServer::spawn_tcp_server(&core_config, app_state.clone()).await;

    // Initialize databases with migrations
    let (diesel_db, sqlx_db) = init_database_pools(&core_config).await?;
    // Run the API
    /*    let api_task = run_api(&core_config, (diesel_db.clone(), sqlx_db), app_state);
     */

    /* GRPC Api */
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(services::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    let user_service =
        UserService::new(Arc::new(UserRepositoryDiesel::new(diesel_db.pool.clone())));
    let user_service = UserServiceServer::new(user_service);

    let auth_service = AuthService::new(
        Arc::new(UserRepositoryDiesel::new(diesel_db.pool)),
        SymmetricKey::generate()?,
    );
    let auth_service = AuthServiceServer::new(auth_service);

    let addr: SocketAddr = "0.0.0.0:50051".parse()?;

    let grpc_web_service = tower::ServiceBuilder::new()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(CorsLayer::very_permissive()) //todo : make this configurable
        .layer(tonic_web::GrpcWebLayer::new())
        .into_inner();

    let grpc_server = move || {
        info!("GRPC server running on {}", addr);
        Server::builder()
            .layer(grpc_web_service)
            .accept_http1(true)
            .add_service(reflection)
            .add_service(user_service)
            .add_service(auth_service)
            .serve(addr)
    };

    // Wait for any task to complete
    tokio::select! {/*
        result = api_task => {
            if let Err(e) = result {
                error!("API error: {}", e);
            }
        }*/
        /*_ = core_task => {
            info!("Core processing task completed");
        }
        _ = tcp_task => {
            info!("TCP server task completed");
        }*/
        _ = grpc_server() => {
            info!("GRPC server task completed");
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
