mod api;
mod config;
mod database;

use crate::api::grpc::auth::controller::AuthController;
use crate::api::grpc::job::controller::JobController;
use crate::api::grpc::orchestrator::controller::OrchestratorController;
use crate::api::grpc::pipeline::controller::PipelineController;
use crate::api::grpc::pipeline::snapshot::controller::PipelineSnapshotController;
use crate::api::grpc::user::controller::UserController;
use crate::config::CoreConfig;
use crate::config::casbin::CASBIN_CONF;
use crate::config::core_config::DatabaseConfig;
use crate::database::{DieselDatabase, set_db_pool};
use anyhow::{Result, anyhow};
use casbin::{CoreApi, Enforcer};
use clap::Parser;
use diesel_adapter::DieselAdapter;
use protocol::services;
use protocol::services::auth_service_server::AuthServiceServer;
use protocol::services::job::job_service_server::JobServiceServer;
use protocol::services::orchestrator::orchestrator_server::OrchestratorServer;
use protocol::services::pipeline::pipeline_server::PipelineServer;
use protocol::services::pipeline::snapshot::pipeline_snapshot_server::PipelineSnapshotServer;
use protocol::services::user_service_server::UserServiceServer;
use protocol::tonic::transport::Server;
use std::sync::Arc;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{Level, info};
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
    set_db_pool(diesel_db.pool.clone());

    // RBAC
    let m = casbin::DefaultModel::from_str(CASBIN_CONF).await?;
    let a = DieselAdapter::with_pool(diesel_db.pool.clone())?;
    let enforcer = Arc::from(Enforcer::new(m, a).await?);

    /* GRPC Api */
    #[cfg(feature = "reflection")]
    let reflection = {
        #[allow(unused_imports)]
        use tonic_reflection::server::Builder as ReflectionBuilder;
        Some(
            ReflectionBuilder::configure()
                .register_encoded_file_descriptor_set(services::FILE_DESCRIPTOR_SET)
                .build_v1alpha()?,
        )
    };

    let user_grpc = UserServiceServer::new(UserController::new());

    let auth_grpc = AuthServiceServer::new(AuthController::new());

    let pipeline_grpc = PipelineServer::new(PipelineController::new(enforcer));

    let pipeline_snapshot_grpc = PipelineSnapshotServer::new(PipelineSnapshotController::new());

    let orchestrator_grpc = OrchestratorServer::with_interceptor(
        OrchestratorController::new(),
        OrchestratorController::check_auth,
    );

    OrchestratorController::set_token("not a good token".into());

    let job_grpc = JobServiceServer::new(JobController::new());

    let grpc_server_fn = move || {
        info!("GRPC server running on {}", grpc_config);
        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Millis),
            );

        let mut server = Server::builder()
            .layer(trace_layer)
            .add_service(user_grpc)
            .add_service(auth_grpc)
            .add_service(pipeline_grpc)
            .add_service(orchestrator_grpc)
            .add_service(pipeline_snapshot_grpc)
            .add_service(job_grpc);
        #[cfg(feature = "reflection")]
        if let Some(reflection) = reflection {
            server = server.add_service(reflection);
        }
        server.serve(grpc_config)
    };

    let _res = grpc_server_fn().await;

    Ok(())
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
