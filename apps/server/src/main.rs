mod api;
mod config;
mod database;

// Internal crate imports
use crate::config::CoreConfig;
use crate::database::DieselDatabase;

// External crate imports
use crate::api::grpc::orchestrator::Orchestrator;
use crate::api::grpc::pipeline::PipelineService;
use crate::api::grpc::pipeline::repo::PipelineRepositoryDiesel;
use crate::api::grpc::pipeline::snapshot::PipelineSnapshotService;
use crate::api::grpc::pipeline::snapshot::repo::PipelineSnapshotRepositoryDiesel;
use crate::api::grpc::pipeline::worker::PipelineWorker;
use crate::api::grpc::{AuthService, BackgroundWorker, UserRepositoryDiesel, UserService};
use crate::config::core_config::DatabaseConfig;
use anyhow::{Result, anyhow};
use clap::Parser;
use pasetors::keys::{Generate, SymmetricKey};
use protocol::services;
use protocol::services::auth_service_server::AuthServiceServer;
use protocol::services::orchestrator::orchestrator_server::OrchestratorServer;
use protocol::services::pipeline::pipeline_server::PipelineServer;
use protocol::services::pipeline::snapshot::pipeline_snapshot_server::PipelineSnapshotServer;
use protocol::services::user_service_server::UserServiceServer;
use protocol::tonic::transport::Server;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
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

fn load_config(args: &Args) -> Result<CoreConfig> {
    if let Some(config_path) = &args.config {
        match CoreConfig::from_toml_file(config_path) {
            Ok(config) => Ok(config),
            Err(e) => {
                panic!("Failed to load configuration from {config_path}: {e}");
            }
        }
    } else {
        let default_config = CoreConfig::default();
        info!(
            "No configuration file provided, using default configuration : {:#?}",
            default_config
        );
        Ok(default_config)
    }
}

async fn init_database_pool(database_config: DatabaseConfig) -> Result<DieselDatabase> {
    // Initialize database and run migrations
    let diesel_db = DieselDatabase::new(&database_config)
        .map_err(|e| anyhow!("Database connection failed: {}", e))?;

    diesel_db
        .run_migrations()
        .map_err(|e| anyhow!("Database migration failed: {}", e))?;

    info!("Database connection established and migrations completed successfully");
    Ok(diesel_db)
}

async fn start_application(core_config: CoreConfig) -> Result<()> {
    let CoreConfig {
        database_config,
        grpc_config,
    } = core_config;

    // Initialize databases with migrations
    let diesel_db = init_database_pool(database_config).await?;

    // Channels
    let (tx_pipeline_service, rx_pipeline_service) = mpsc::channel(100);

    let (tx_shutdown, rx_shutdown) = tokio::sync::watch::channel(false);

    /* GRPC Api */
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(services::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    let user_service =
        UserService::new(Arc::new(UserRepositoryDiesel::new(diesel_db.pool.clone())));
    let user_grpc = UserServiceServer::new(user_service);

    let auth_service = AuthService::new(
        Arc::new(UserRepositoryDiesel::new(diesel_db.pool.clone())),
        SymmetricKey::generate()?,
    );
    let auth_grpc = AuthServiceServer::new(auth_service);

    let pipeline_repo = PipelineRepositoryDiesel::new(diesel_db.pool.clone());
    let pipeline_service = PipelineService::new(Arc::new(pipeline_repo.clone()));
    let pipeline_worker = PipelineWorker::new(Arc::new(pipeline_repo), rx_pipeline_service);
    let pipeline_grpc = PipelineServer::new(pipeline_service);

    let pipeline_snapshot_service = PipelineSnapshotService::new(
        Arc::new(PipelineSnapshotRepositoryDiesel::new(diesel_db.pool)),
        tx_pipeline_service,
    );
    let pipeline_snapshot_grpc = PipelineSnapshotServer::new(pipeline_snapshot_service);

    let orchestrator = Orchestrator::default();
    let orchestrator_grpc = OrchestratorServer::new(orchestrator.clone());

    /*let cert = fs::read("certs/origin-cert.pem").await?;
    let key = fs::read("certs/origin-key.pem").await?;

    let identity = Identity::from_pem(cert, key);*/

    let grpc_web_service = tower::ServiceBuilder::new()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(CorsLayer::very_permissive()) //todo : make this configurable
        .layer(tonic_web::GrpcWebLayer::new())
        .into_inner();

    let mut threads = JoinSet::new();
    threads.spawn(BackgroundWorker::spawn_worker(
        orchestrator,
        rx_shutdown.clone(),
    ));
    threads.spawn(BackgroundWorker::spawn_worker(
        pipeline_worker,
        rx_shutdown.clone(),
    ));

    let grpc_server_fn = move || {
        info!("GRPC server running on {}", grpc_config);
        Server::builder()
            /*.tls_config(ServerTlsConfig::new().identity(identity)).unwrap()*/
            .layer(grpc_web_service)
            .accept_http1(true)
            .add_service(reflection)
            .add_service(user_grpc)
            .add_service(auth_grpc)
            .add_service(pipeline_grpc)
            .add_service(orchestrator_grpc)
            .add_service(pipeline_snapshot_grpc)
            .serve_with_shutdown(grpc_config, async move {
                tokio::signal::ctrl_c().await.ok();
                let _ = tx_shutdown.send(true);
            })
    };

    let _res = grpc_server_fn().await;
    let _res = threads.join_all().await;

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
