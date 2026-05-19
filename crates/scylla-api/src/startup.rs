use crate::config::CoreConfig;
use crate::error::StartupError;
use hermes_broker_client::Publisher;
use http::{HeaderName, HeaderValue, Method};
use scylla_core::application::{
    AgentUseCases, AuthUseCases, JobLogStreamUseCase, JobLogUseCases, JobUseCases,
    OrganizationUseCases, PermissionUseCases, PipelineUseCases, ProjectUseCases, UserUseCases,
};
use scylla_core::infrastructure::{
    Argon2HashService, CasbinPermissionService, HermesJobLogStream, PgAgentRepository,
    PgJobLogRepository, PgJobRepository, PgOrganizationRepository, PgPipelineRepository,
    PgProjectRepository, PgSessionRepository, PgUserOrganizationRepository,
    PgUserProjectRepository, PgUserRepository,
};
use sqlx::PgPool;
use sqlx_adapter::SqlxAdapter;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

// ── Concrete type aliases ──────────────────────────────────────────────

pub type SharedAuthUc = Arc<AuthUseCases<PgUserRepository, PgSessionRepository, Argon2HashService>>;
pub type SharedUserUc = Arc<UserUseCases<PgUserRepository, Argon2HashService>>;
pub type SharedOrgUc = Arc<
    OrganizationUseCases<PgOrganizationRepository, PgUserOrganizationRepository, PgUserRepository>,
>;
pub type SharedProjectUc =
    Arc<ProjectUseCases<PgProjectRepository, PgUserProjectRepository, PgUserRepository>>;
pub type SharedPipelineUc = Arc<PipelineUseCases<PgPipelineRepository, PgProjectRepository>>;
pub type SharedJobUc = Arc<JobUseCases<PgJobRepository>>;
pub type SharedJobLogUc = Arc<JobLogUseCases<PgJobLogRepository>>;
pub type SharedJobLogStreamUc = Arc<JobLogStreamUseCase<PgJobLogRepository, HermesJobLogStream>>;
pub type SharedAgentUc = Arc<AgentUseCases<PgAgentRepository>>;
pub type SharedPermissionUc = Arc<PermissionUseCases<CasbinPermissionService>>;

// ── Services container ─────────────────────────────────────────────────

pub struct Services {
    pub db: PgPool,
    pub auth_uc: SharedAuthUc,
    pub user_uc: SharedUserUc,
    pub org_uc: SharedOrgUc,
    pub project_uc: SharedProjectUc,
    pub pipeline_uc: SharedPipelineUc,
    pub job_uc: SharedJobUc,
    pub job_log_uc: SharedJobLogUc,
    pub job_log_stream_uc: SharedJobLogStreamUc,
    pub agent_uc: SharedAgentUc,
    pub permission_uc: SharedPermissionUc,
    pub permission_checker: Arc<CasbinPermissionService>,
    pub session_repo: Arc<PgSessionRepository>,
    pub broker_publisher: Arc<Publisher>,
}

pub async fn init_services(config: &CoreConfig) -> Result<Services, StartupError> {
    let db = scylla_core::infrastructure::init_db(&config.database).await?;

    let user_repo = Arc::new(PgUserRepository::new(db.clone()));
    let session_repo = Arc::new(PgSessionRepository::new(db.clone()));
    let org_repo = Arc::new(PgOrganizationRepository::new(db.clone()));
    let project_repo = Arc::new(PgProjectRepository::new(db.clone()));
    let pipeline_repo = Arc::new(PgPipelineRepository::new(db.clone()));
    let job_repo = Arc::new(PgJobRepository::new(db.clone()));
    let job_log_repo = Arc::new(PgJobLogRepository::new(db.clone()));
    let agent_repo = Arc::new(PgAgentRepository::new(db.clone()));
    let user_org_repo = Arc::new(PgUserOrganizationRepository::new(db.clone()));
    let user_project_repo = Arc::new(PgUserProjectRepository::new(db.clone()));
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
    let pipeline_uc = Arc::new(PipelineUseCases::new(
        pipeline_repo.clone(),
        project_repo.clone(),
    ));
    let job_uc = Arc::new(JobUseCases::new(job_repo.clone()));
    let job_log_uc = Arc::new(JobLogUseCases::new(job_log_repo.clone()));
    let agent_uc = Arc::new(AgentUseCases::new(agent_repo.clone()));
    // The sqlx Casbin adapter creates its own `casbin_rule` table on first use.
    let casbin_adapter = SqlxAdapter::new_with_pool(db.clone())
        .await
        .map_err(|e| StartupError::CasbinAdapter(e.to_string()))?;
    let mut casbin_service = CasbinPermissionService::new(casbin_adapter)
        .await
        .map_err(|e| StartupError::CasbinService(e.to_string()))?;

    if let Some(cfg) = &config.bootstrap {
        crate::bootstrap::bootstrap_admin(&user_uc, &mut casbin_service, cfg).await?;
    }

    let permission_checker = Arc::new(casbin_service);
    let permission_uc = Arc::new(PermissionUseCases::new(permission_checker.clone()));

    // Connect to Hermes broker
    let broker_channel = hermes_broker_client::connect(&config.broker.url, None)
        .await
        .map_err(|e| StartupError::BrokerConnect {
            url: config.broker.url.clone(),
            message: e.to_string(),
        })?;
    tracing::info!(url = %config.broker.url, "connected to hermes broker");
    let broker_publisher = Arc::new(Publisher::new(broker_channel.clone()));
    let job_log_stream_port = Arc::new(HermesJobLogStream::new(broker_channel));
    let job_log_stream_uc = Arc::new(JobLogStreamUseCase::new(
        job_log_repo.clone(),
        job_log_stream_port,
    ));

    Ok(Services {
        db,
        auth_uc,
        user_uc,
        org_uc,
        project_uc,
        pipeline_uc,
        job_uc,
        job_log_uc,
        job_log_stream_uc,
        agent_uc,
        permission_uc,
        permission_checker,
        session_repo,
        broker_publisher,
    })
}

// ── CORS builder ───────────────────────────────────────────────────────

pub fn build_cors_layer(cors: &crate::config::CorsConfig) -> CorsLayer {
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

    let expose_headers: Vec<HeaderName> = cors
        .expose_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();
    layer = layer.expose_headers(expose_headers);

    layer
}

// ── Graceful shutdown signal ───────────────────────────────────────────

pub async fn shutdown_signal() {
    let ctrl_c = async {
        // INVARIANT: Ctrl+C handler installation cannot fail at startup on supported platforms.
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        // INVARIANT: SIGTERM handler installation cannot fail at startup on supported platforms.
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::info!("Received Ctrl+C, shutting down"),
        () = terminate => tracing::info!("Received SIGTERM, shutting down"),
    }
}

// ── gRPC server ────────────────────────────────────────────────────────

#[cfg(feature = "grpc")]
pub async fn start_grpc(config: &CoreConfig, services: &Services) -> Result<(), StartupError> {
    use scylla_api::{
        AgentHandler, AuthHandler, JobHandler, OrganizationHandler, PermissionHandler,
        PipelineHandler, ProjectHandler, UserHandler, auth_interceptor::AuthInterceptor,
    };
    use scylla_protocol::services::{
        agent::agent_service_server::AgentServiceServer,
        auth::auth_service_server::AuthServiceServer, job::job_service_server::JobServiceServer,
        organization::organization_service_server::OrganizationServiceServer,
        permission::permission_service_server::PermissionServiceServer,
        pipeline::pipeline_service_server::PipelineServiceServer,
        project::project_service_server::ProjectServiceServer,
        user::user_service_server::UserServiceServer,
    };
    use tonic::transport::Server;
    use tonic_async_interceptor::async_interceptor;
    use tonic_web::GrpcWebLayer;
    use tower::ServiceBuilder;
    use tower_http::trace::TraceLayer;

    let auth_handler = AuthHandler::new(services.auth_uc.clone());
    let user_handler = UserHandler::new(
        services.user_uc.clone(),
        services.permission_checker.clone(),
    );
    let org_handler =
        OrganizationHandler::new(services.org_uc.clone(), services.permission_checker.clone());
    let project_handler = ProjectHandler::new(
        services.project_uc.clone(),
        services.permission_checker.clone(),
    );
    let pipeline_handler = PipelineHandler::new(
        services.pipeline_uc.clone(),
        services.job_uc.clone(),
        services.permission_checker.clone(),
        services.broker_publisher.clone(),
    );
    let job_handler = JobHandler::new(
        services.job_uc.clone(),
        services.job_log_uc.clone(),
        services.job_log_stream_uc.clone(),
        services.permission_checker.clone(),
    );
    let agent_handler = AgentHandler::new(
        services.agent_uc.clone(),
        services.permission_checker.clone(),
    );
    let permission_handler = PermissionHandler::new(
        services.permission_uc.clone(),
        services.permission_checker.clone(),
    );

    let auth_interceptor = async_interceptor(AuthInterceptor::new(services.session_repo.clone()));
    let cors_layer = build_cors_layer(&config.cors);

    tracing::info!("gRPC server listening on {}", config.grpc.address);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(scylla_protocol::services::FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .map_err(|e| StartupError::Reflection(e.to_string()))?;

    let auth_service = AuthServiceServer::new(auth_handler);

    let user_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(UserServiceServer::new(user_handler));

    let org_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(OrganizationServiceServer::new(org_handler));

    let project_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(ProjectServiceServer::new(project_handler));

    let pipeline_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(PipelineServiceServer::new(pipeline_handler));

    let job_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(JobServiceServer::new(job_handler));

    let agent_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(AgentServiceServer::new(agent_handler));

    let permission_service = ServiceBuilder::new()
        .layer(auth_interceptor)
        .service(PermissionServiceServer::new(permission_handler));

    Server::builder()
        .accept_http1(true)
        .layer(TraceLayer::new_for_grpc())
        .layer(cors_layer)
        .layer(GrpcWebLayer::new())
        .add_service(reflection)
        .add_service(auth_service)
        .add_service(user_service)
        .add_service(org_service)
        .add_service(project_service)
        .add_service(pipeline_service)
        .add_service(job_service)
        .add_service(agent_service)
        .add_service(permission_service)
        .serve_with_shutdown(config.grpc.address, shutdown_signal())
        .await?;

    // Gracefully close the PostgreSQL pool so in-flight queries can complete.
    scylla_core::infrastructure::close_db(&services.db).await;

    Ok(())
}
