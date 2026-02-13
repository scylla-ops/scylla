mod config;

use application::{AuthUseCases, OrganizationUseCases, ProjectUseCases, UserUseCases};
use config::{BootstrapConfig, CoreConfig};
use infrastructure::{
    Argon2HashService, SurrealOrganizationRepository, SurrealProjectRepository,
    SurrealSessionRepository, SurrealUserOrganizationRepository, SurrealUserProjectRepository,
    SurrealUserRepository,
};
use interfaces::{
    AuthHandler, OrganizationHandler, ProjectHandler, UserHandler, auth_interceptor,
    services::{
        auth::auth_service_server::AuthServiceServer,
        organization::organization_service_server::OrganizationServiceServer,
        project::project_service_server::ProjectServiceServer,
        user::user_service_server::UserServiceServer,
    },
};

use anyhow::{Context, Result};
use clap::Parser;
use domain::errors::DomainError;
use domain::value_objects::user::{Password, UserName};
use http::{HeaderName, HeaderValue, Method};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use tonic::transport::Server;
use tower_http::cors::CorsLayer;

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
        log::info!("No configuration file provided, using defaults");
        Ok(CoreConfig::default())
    }
}

fn build_cors_layer(cors: &config::CorsConfig) -> CorsLayer {
    let mut layer = CorsLayer::new();

    // Origins
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

    // Methods
    let methods: Vec<Method> = cors
        .allow_methods
        .iter()
        .filter_map(|m| m.parse().ok())
        .collect();
    layer = layer.allow_methods(methods);

    // Headers
    let headers: Vec<HeaderName> = cors
        .allow_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();
    layer = layer.allow_headers(headers);

    // Max age
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
            log::info!(
                "Bootstrap user '{}' created (id: {})",
                bootstrap.username,
                user.id()
            );
        }
        Err(DomainError::Conflict(_)) => {
            log::debug!(
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
    env_logger::init();
    let args = Args::parse();

    if args.print_example_config {
        CoreConfig::print_example();
        return;
    }

    if let Err(e) = run(args).await {
        log::error!("Application error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<()> {
    let config = load_config(&args)?;
    log::debug!("Configuration: {:#?}", config);

    let db_config = &config.database;

    // Connect to SurrealDB
    let db: Surreal<Any> = Surreal::init();
    db.connect(&db_config.url)
        .await
        .with_context(|| format!("Failed to connect to database at {}", db_config.url))?;
    db.signin(surrealdb::opt::auth::Root {
        username: &db_config.username,
        password: &db_config.password,
    })
    .await
    .context("Failed to authenticate with database")?;
    db.use_ns(&db_config.namespace)
        .use_db(&db_config.database)
        .await
        .context("Failed to select namespace/database")?;
    let db = Arc::new(db);

    // Repositories
    let user_repo = Arc::new(SurrealUserRepository::new(db.clone()));
    let session_repo = Arc::new(SurrealSessionRepository::new(db.clone()));
    let org_repo = Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let project_repo = Arc::new(SurrealProjectRepository::new(db.clone()));
    let user_org_repo = Arc::new(SurrealUserOrganizationRepository::new(db.clone()));
    let user_project_repo = Arc::new(SurrealUserProjectRepository::new(db.clone()));
    let hash_service = Arc::new(Argon2HashService::new());

    // Use cases
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

    // Bootstrap
    if let Some(bootstrap) = &config.bootstrap {
        bootstrap_admin(&user_uc, bootstrap).await?;
    }

    // Handlers
    let auth_handler = AuthHandler::new(auth_uc);
    let user_handler = UserHandler::new(user_uc);
    let org_handler = OrganizationHandler::new(org_uc);
    let project_handler = ProjectHandler::new(project_uc);

    // Auth interceptor
    let interceptor = auth_interceptor(session_repo);

    // CORS
    let cors_layer = build_cors_layer(&config.cors);

    // Serve
    log::info!("gRPC server listening on {}", config.grpc.address);

    Server::builder()
        .layer(cors_layer)
        .add_service(AuthServiceServer::new(auth_handler))
        .add_service(UserServiceServer::with_interceptor(
            user_handler,
            interceptor.clone(),
        ))
        .add_service(OrganizationServiceServer::with_interceptor(
            org_handler,
            interceptor.clone(),
        ))
        .add_service(ProjectServiceServer::with_interceptor(
            project_handler,
            interceptor,
        ))
        .serve(config.grpc.address)
        .await?;

    Ok(())
}
