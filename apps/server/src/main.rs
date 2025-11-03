use protocol::services;
use protocol::services::auth::auth_service_server::AuthServiceServer;
use protocol::services::job::job_service_server::JobServiceServer;
use protocol::services::organization::organization_service_server::OrganizationServiceServer;
use protocol::services::pipeline::pipeline_server::PipelineServer;
use protocol::services::project::project_service_server::ProjectServiceServer;
use protocol::services::user::user_service_server::UserServiceServer;
use protocol::tonic::transport::Server;

use scylla_core::config::CoreConfig;
use scylla_core::infrastructure::database::{apply_migrations, db, init_db, login};
use scylla_core::presentation::grpc::handlers::{
    AuthHandler, JobHandler, OrganizationHandler, PipelineHandler, ProjectHandler, UserHandler,
};
use scylla_core::shared::di::AppContainer;

use anyhow::{Context, Result};
use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi};
use clap::Parser;
use std::sync::Arc;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{Level, info, warn};
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
        CoreConfig::from_toml_file(config_path)
            .with_context(|| format!("Failed to load configuration from {}", config_path))
    } else {
        let default_config = CoreConfig::default();
        info!(
            "No configuration file provided, using default configuration : {:#?}",
            default_config
        );
        Ok(default_config)
    }
}

async fn init_rbac_enforcer(
    db: Arc<surrealdb::Surreal<surrealdb::engine::any::Any>>,
    rbac_config: &scylla_core::config::RbacConfig,
) -> Result<Enforcer> {
    // Try to read model from configured path, fall back to embedded default if not found
    let model_text = match std::fs::read_to_string(&rbac_config.model_path) {
        Ok(content) => {
            info!("Loaded Casbin model from: {}", rbac_config.model_path);
            content
        }
        Err(_) => {
            // Fall back to embedded default if configured path doesn't exist
            info!(
                "Model file not found at {}, using embedded default",
                rbac_config.model_path
            );
            include_str!("../casbin/model.conf").to_string()
        }
    };
    let model = DefaultModel::from_str(&model_text).await?;

    // Use SurrealDB adapter for persistent policy storage
    let adapter = surreal_casbin_adapter::SurrealAdapter::new(db, "casbin_rules");
    let mut enforcer = Enforcer::new(model, adapter).await?;

    // Bootstrap default policies if none exist
    let policies = enforcer.get_all_policy();
    if policies.is_empty() {
        info!("No existing policies found, bootstrapping default RBAC policies");

        let bootstrap_policies = [
            ("admin", "*", "organizations", "create"),
            ("admin", "*", "organizations", "read"),
            ("admin", "*", "organizations", "update"),
            ("admin", "*", "organizations", "delete"),
            ("admin", "*", "users", "create"),
            ("admin", "*", "users", "read"),
            ("admin", "*", "users", "update"),
            ("admin", "*", "users", "delete"),
            ("user", "*", "organizations", "read"),
        ];

        for (role, domain, resource, action) in bootstrap_policies {
            enforcer
                .add_policy(vec![
                    role.to_string(),
                    domain.to_string(),
                    resource.to_string(),
                    action.to_string(),
                ])
                .await?;
        }

        info!("Default RBAC policies bootstrapped successfully");
    } else {
        info!(
            "Loaded {} existing RBAC policies from database",
            policies.len()
        );
    }

    info!("RBAC enforcer initialized with SurrealDB adapter");
    Ok(enforcer)
}

async fn bootstrap_admin_user(
    container: &scylla_core::shared::di::AppContainer,
    bootstrap_config: &scylla_core::config::BootstrapConfig,
) -> Result<()> {
    use scylla_core::domain::entities::User;
    use scylla_core::domain::value_objects::{Password, Username};

    // Check if bootstrap is enabled
    if !bootstrap_config.enabled {
        info!("Bootstrap admin user creation is disabled");
        return Ok(());
    }

    // Check if any admins exist in the global domain ("*")
    let rbac_enforcer = container.rbac_enforcer();
    let admin_users = rbac_enforcer
        .get_users_for_role("admin", "*")
        .await
        .with_context(|| "Failed to check for existing admin users")?;

    if !admin_users.is_empty() {
        info!(
            "Admin user(s) already exist (count: {}), skipping bootstrap",
            admin_users.len()
        );
        return Ok(());
    }

    // No admins exist - proceed with bootstrap
    info!(
        "No admin users found, creating bootstrap admin user: {}",
        bootstrap_config.username
    );

    // Create username value object
    let username = Username::try_from(bootstrap_config.username.clone())
        .with_context(|| "Failed to create username for bootstrap admin")?;

    // Create password value object
    let password = Password::new(bootstrap_config.password.clone())
        .with_context(|| "Failed to create password for bootstrap admin")?;

    // Hash the password
    let password_hasher = container.password_hasher();
    let password_hash = password_hasher
        .hash(&password)
        .await
        .with_context(|| "Failed to hash bootstrap admin password")?;

    // Create the user entity
    let user = User::create(username, password_hash);

    // Save to repository
    let user_repo = container.user_repo();
    let created_user = user_repo
        .create(&user)
        .await
        .with_context(|| "Failed to create bootstrap admin user in database")?;

    // Assign admin role in global domain
    rbac_enforcer
        .add_role_for_user(created_user.id(), "admin", "*")
        .await
        .with_context(|| "Failed to assign admin role to bootstrap user")?;

    info!(
        "Bootstrap admin user created successfully with ID: {}",
        created_user.id().as_str()
    );
    warn!("SECURITY WARNING: Change the bootstrap admin password immediately after first login!");

    Ok(())
}

async fn start_application(core_config: CoreConfig) -> Result<()> {
    let CoreConfig {
        database_config,
        grpc_config,
        auth_config,
        rbac_config,
        bootstrap_config,
    } = core_config;

    // Initialize database connection
    init_db(
        &database_config.url,
        &database_config.namespace,
        &database_config.database,
    )
    .await?;

    login(&database_config.username, &database_config.password)
        .await
        .with_context(|| "Failed to login to database")?;

    apply_migrations(db()?).await?;

    // Initialize RBAC enforcer with SurrealDB adapter
    let enforcer = init_rbac_enforcer(db()?, &rbac_config).await?;

    // Initialize dependency injection container
    let container = Arc::new(AppContainer::new(db()?, enforcer, &auth_config)?);

    // Bootstrap admin user if configured
    bootstrap_admin_user(&container, &bootstrap_config).await?;

    // Create gRPC handlers
    let user_handler = UserHandler::new(container.clone());
    let auth_handler = AuthHandler::new(container.clone());
    let organization_handler = OrganizationHandler::new(container.clone());
    let project_handler = ProjectHandler::new(container.clone());
    let pipeline_handler = PipelineHandler::new(container.clone());
    let job_handler = JobHandler::new(container.clone());

    // Create auth interceptor
    let interceptor =
        scylla_core::presentation::grpc::middleware::auth_interceptor(container.clone());

    // Create gRPC servers
    let auth_grpc = AuthServiceServer::new(auth_handler);
    // let user_grpc = UserServiceServer::new(user_handler);

    let user_grpc = UserServiceServer::with_interceptor(user_handler, interceptor.clone());
    let organization_grpc =
        OrganizationServiceServer::with_interceptor(organization_handler, interceptor.clone());
    let project_grpc = ProjectServiceServer::with_interceptor(project_handler, interceptor.clone());
    let pipeline_grpc = PipelineServer::with_interceptor(pipeline_handler, interceptor.clone());
    let job_grpc = JobServiceServer::with_interceptor(job_handler, interceptor.clone());

    // Setup reflection
    #[cfg(feature = "reflection")]
    let reflection = {
        use tonic_reflection::server::Builder as ReflectionBuilder;
        Some(
            ReflectionBuilder::configure()
                .register_encoded_file_descriptor_set(services::FILE_DESCRIPTOR_SET)
                .build_v1alpha()?,
        )
    };

    // Start gRPC server
    info!("GRPC server running on {}", grpc_config.address);
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
        .add_service(job_grpc);

    #[cfg(feature = "reflection")]
    if let Some(reflection) = reflection {
        server = server.add_service(reflection);
    }

    server.serve(grpc_config.address).await?;

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
