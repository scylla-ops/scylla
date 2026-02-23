mod config;

use application::{AuthUseCases, OrganizationUseCases, ProjectUseCases, UserUseCases};
use config::{BootstrapConfig, CoreConfig};
use infrastructure::{
    Argon2HashService, SurrealOrganizationRepository, SurrealProjectRepository,
    SurrealSessionRepository, SurrealUserOrganizationRepository, SurrealUserProjectRepository,
    SurrealUserRepository,
};
use interfaces::{AuthHandler, OrganizationHandler, ProjectHandler, UserHandler};
use protocol::services::{
    auth::auth_service_server::AuthServiceServer,
    organization::organization_service_server::OrganizationServiceServer,
    project::project_service_server::ProjectServiceServer,
    user::user_service_server::UserServiceServer,
};

use anyhow::{Context, Result};
use clap::Parser;
use domain::errors::DomainError;
use domain::value_objects::user::{Password, UserName};
use http::{HeaderName, HeaderValue, Method};
use interfaces::auth_interceptor::AuthInterceptor;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use tonic::transport::Server;
use tonic_async_interceptor::async_interceptor;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Parser, Debug)]
#[command(author, version, about = "Scylla Core gRPC server")]
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
        CoreConfig::from_file(config_path)
            .with_context(|| format!("Failed to load configuration from {}", config_path))
    } else {
        tracing::info!("No configuration file provided, using defaults");
        Ok(CoreConfig::default())
    }
}

fn build_cors_layer(cors: &config::CorsConfig) -> CorsLayer {
    let mut layer = CorsLayer::new();

    if cors.allow_origins.iter().any(|o| o == "*") {
        layer = layer.allow_origin(tower_http::cors::Any);
    } else {
        let origins: Vec<HeaderValue> = cors
            .allow_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        layer = layer.allow_origin(origins);
    }

    let methods: Vec<Method> = cors
        .allow_methods
        .iter()
        .filter_map(|m| m.parse().ok())
        .collect();
    layer = layer.allow_methods(methods);

    let headers: Vec<HeaderName> = cors
        .allow_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();
    layer = layer.allow_headers(headers);

    layer = layer.max_age(std::time::Duration::from_secs(cors.max_age_seconds));

    layer
}

async fn bootstrap_admin<U: domain::ports::UserRepository, H: domain::ports::HashService>(
    user_uc: &UserUseCases<U, H>,
    bootstrap: &BootstrapConfig,
) -> Result<()> {
    let username = UserName::new(&bootstrap.username).context("Invalid bootstrap username")?;
    let password = Password::new(&bootstrap.password).context("Invalid bootstrap password")?;

    match user_uc.create(username, password).await {
        Ok(user) => {
            tracing::info!(
                "Bootstrap user '{}' created (id: {})",
                bootstrap.username,
                user.id()
            );
        }
        Err(DomainError::Conflict(_)) => {
            tracing::debug!(
                "Bootstrap user '{}' already exists, skipping",
                bootstrap.username
            );
        }
        Err(e) => return Err(e).context("Failed to bootstrap admin user"),
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
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

    let db_config = &config.database;

    let db: Surreal<Any> = Surreal::init();
    db.connect(&db_config.url)
        .await
        .with_context(|| format!("Failed to connect to database at {}", db_config.url))?;
    db.signin(surrealdb::opt::auth::Root {
        username: db_config.username.clone(),
        password: db_config.password.clone(),
    })
    .await
    .context("Failed to authenticate with database")?;
    db.use_ns(&db_config.namespace)
        .use_db(&db_config.database)
        .await
        .context("Failed to select namespace/database")?;

    db.query(
        "
        DEFINE TABLE IF NOT EXISTS users SCHEMALESS;
        DEFINE TABLE IF NOT EXISTS sessions SCHEMALESS;
        DEFINE TABLE IF NOT EXISTS organizations SCHEMALESS;
        DEFINE TABLE IF NOT EXISTS projects SCHEMALESS;
        DEFINE TABLE IF NOT EXISTS user_organization SCHEMALESS;
        DEFINE TABLE IF NOT EXISTS user_project SCHEMALESS;
    ",
    )
    .await
    .context("Failed to initialize database tables")?
    .check()
    .context("Database schema init returned an error")?;
    
    let user_repo = Arc::new(SurrealUserRepository::new(db.clone()));
    let session_repo = Arc::new(SurrealSessionRepository::new(db.clone()));
    let org_repo = Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let project_repo = Arc::new(SurrealProjectRepository::new(db.clone()));
    let user_org_repo = Arc::new(SurrealUserOrganizationRepository::new(db.clone()));
    let user_project_repo = Arc::new(SurrealUserProjectRepository::new(db.clone()));
    let hash_service = Arc::new(Argon2HashService::new());

    let auth_uc = Arc::new(AuthUseCases::new(
        user_repo.clone(),
        session_repo.clone(),
        hash_service.clone(),
    ));
    let user_uc = Arc::new(UserUseCases::new(user_repo.clone(), hash_service.clone()));
    let org_uc = Arc::new(OrganizationUseCases::new(
        org_repo.clone(),
        user_org_repo.clone(),
        user_repo.clone(),
    ));
    let project_uc = Arc::new(ProjectUseCases::new(
        project_repo.clone(),
        user_project_repo.clone(),
        user_repo.clone(),
    ));

    if let Some(bootstrap) = &config.bootstrap {
        bootstrap_admin(&user_uc, bootstrap).await?;
    }

    let auth_handler = AuthHandler::new(auth_uc);
    let user_handler = UserHandler::new(user_uc);
    let org_handler = OrganizationHandler::new(org_uc);
    let project_handler = ProjectHandler::new(project_uc);

    let auth_interceptor = async_interceptor(AuthInterceptor::new(session_repo.clone()));
    let cors_layer = build_cors_layer(&config.cors);

    tracing::info!("gRPC server listening on {}", config.grpc.address);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(protocol::services::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    let auth_service = AuthServiceServer::new(auth_handler);

    let user_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(UserServiceServer::new(user_handler));

    let org_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(OrganizationServiceServer::new(org_handler));

    let project_service = ServiceBuilder::new()
        .layer(auth_interceptor)
        .service(ProjectServiceServer::new(project_handler));

    Server::builder()
        .layer(TraceLayer::new_for_grpc())
        .layer(cors_layer)
        .add_service(reflection)
        .add_service(auth_service)
        .add_service(user_service)
        .add_service(org_service)
        .add_service(project_service)
        .serve(config.grpc.address)
        .await?;

    Ok(())
}
