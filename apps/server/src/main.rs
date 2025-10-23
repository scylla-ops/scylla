mod api;
mod config;
mod database;

use crate::api::grpc::auth::controller::AuthController;
use crate::api::grpc::job::controller::JobController;
use crate::api::grpc::orchestrator::controller::OrchestratorController;
use crate::api::grpc::organization::controller::OrganizationController;
use crate::api::grpc::pipeline::controller::PipelineController;
use crate::api::grpc::pipeline::snapshot::controller::PipelineSnapshotController;
use crate::api::grpc::project::controller::ProjectController;
use crate::api::grpc::user::controller::UserController;
use crate::config::CoreConfig;
use crate::database::{apply_migrations, db, init_db, login};
use anyhow::Result;
use clap::Parser;
use protocol::services;
use protocol::services::auth_service_server::AuthServiceServer;
use protocol::services::job::job_service_server::JobServiceServer;
use protocol::services::orchestrator::orchestrator_server::OrchestratorServer;
use protocol::services::organization::organization_service_server::OrganizationServiceServer;
use protocol::services::pipeline::pipeline_server::PipelineServer;
use protocol::services::pipeline::snapshot::pipeline_snapshot_server::PipelineSnapshotServer;
use protocol::services::project::project_service_server::ProjectServiceServer;
use protocol::services::user_service_server::UserServiceServer;
use protocol::tonic::transport::Server;
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

async fn start_application(core_config: CoreConfig) -> Result<()> {
    let CoreConfig {
        database_config,
        grpc_config,
    } = core_config;

    // init database conn
    init_db(
        &database_config.url,
        &database_config.namespace,
        &database_config.database,
    )
    .await?;

    login(&database_config.username, &database_config.password)
        .await
        .expect("Failed to login");

    apply_migrations(db()).await;

    // RBAC enforcer
    crate::api::grpc::rbac::init_enforcer().await?;

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

    let user_grpc = UserServiceServer::new(UserController);
    let auth_grpc = AuthServiceServer::new(AuthController);

    let organization_grpc = OrganizationServiceServer::new(OrganizationController);

    let project_grpc = ProjectServiceServer::new(ProjectController);

    let pipeline_grpc = PipelineServer::new(PipelineController);

    let pipeline_snapshot_grpc = PipelineSnapshotServer::new(PipelineSnapshotController);

    let orchestrator_grpc = OrchestratorServer::with_interceptor(
        OrchestratorController,
        OrchestratorController::check_auth,
    );

    OrchestratorController::set_token("not a good token".into());

    let job_grpc = JobServiceServer::new(JobController);

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
            .accept_http1(true)
            .layer(trace_layer)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_service(user_grpc)
            .add_service(auth_grpc)
            .add_service(organization_grpc)
            .add_service(project_grpc)
            .add_service(pipeline_grpc)
            .add_service(pipeline_snapshot_grpc)
            .add_service(job_grpc)
            .add_service(orchestrator_grpc);
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
