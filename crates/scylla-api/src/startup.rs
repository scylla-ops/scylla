use crate::config::CoreConfig;
use crate::error::StartupError;
use http::{HeaderName, HeaderValue, Method};
use scylla_core::application::{
    AppTokenUseCases, AppUseCases, AuditLog, AuthUseCases, DispatchUseCases,
    GrantUseCases, JobLogStreamUseCase, JobLogUseCases, JobUseCases, OrganizationUseCases,
    PipelineUseCases, PolicyUseCases, ProjectUseCases, UserRoleUseCases, UserUseCases,
    WorkerUseCases,
};
#[cfg(feature = "signup")]
use scylla_core::application::SignupUseCases;
#[cfg(feature = "mail")]
use scylla_core::application::{Mailer, NoopMailer};
#[cfg(feature = "mail")]
use scylla_core::infrastructure::LettreMailer;
#[cfg(feature = "invitations")]
use scylla_core::application::InvitationUseCases;
#[cfg(feature = "invitations")]
use scylla_core::infrastructure::PgInvitationRepository;
#[cfg(feature = "oauth-github")]
use scylla_core::application::OAuthUseCases;
#[cfg(feature = "oauth-github")]
use scylla_core::infrastructure::{GitHubOAuthProvider, PgOAuthIdentityRepository};
// PgSignupRepository (core, always compiled) backs both signup and OAuth account
// provisioning, so it's imported whenever either feature is on.
#[cfg(all(not(feature = "signup"), feature = "oauth-github"))]
use scylla_core::infrastructure::PgSignupRepository;
use scylla_core::infrastructure::{
    Argon2HashService, CedarPermissionService, InMemoryJobLogStream, InMemoryWorkerRegistry,
    PgAppRepository, PgAppTokenRepository, PgAuditLog, PgAuthzEntityProvider,
    PgGrantRepository, PgJobLogRepository, PgJobRepository, PgOrganizationRepository,
    PgPipelineRepository, PgPolicyRepository, PgProjectRepository, PgSessionRepository,
    PgUserOrganizationRepository, PgUserProjectRepository, PgUserRepository, PgUserRoleRepository,
    PgWorkerRepository,
};
#[cfg(feature = "signup")]
use scylla_core::infrastructure::PgSignupRepository;
use sqlx::PgPool;
use std::future::Future;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

// ── Concrete type aliases ──────────────────────────────────────────────

pub type PermissionChecker = CedarPermissionService<PgAuthzEntityProvider>;
pub type SharedPermissionChecker = Arc<PermissionChecker>;
pub type SharedGrantUc =
    Arc<GrantUseCases<PgGrantRepository, PermissionChecker, PermissionChecker>>;
pub type SharedPolicyUc =
    Arc<PolicyUseCases<PgPolicyRepository, PermissionChecker, PermissionChecker>>;
pub type SharedRoleUc = Arc<UserRoleUseCases<PgUserRoleRepository, PermissionChecker>>;

pub type SharedAuthUc =
    Arc<AuthUseCases<PgUserRepository, PgSessionRepository, Argon2HashService>>;
#[cfg(feature = "signup")]
pub type SharedSignupUc = Arc<
    SignupUseCases<PgSignupRepository, PgSessionRepository, Argon2HashService, PermissionChecker>,
>;
#[cfg(feature = "invitations")]
pub type SharedInvitationUc = Arc<
    InvitationUseCases<
        PgInvitationRepository,
        PermissionChecker,
        PgOrganizationRepository,
        PgUserRepository,
        Argon2HashService,
        PgSessionRepository,
        PermissionChecker,
    >,
>;
#[cfg(feature = "oauth-github")]
pub type SharedOAuthUc = Arc<
    OAuthUseCases<
        GitHubOAuthProvider,
        PgOAuthIdentityRepository,
        PgSignupRepository,
        PgUserRepository,
        PgSessionRepository,
        Argon2HashService,
        PermissionChecker,
    >,
>;
pub type SharedUserUc = Arc<UserUseCases<PgUserRepository, Argon2HashService, PermissionChecker>>;
pub type SharedOrgUc = Arc<
    OrganizationUseCases<
        PgOrganizationRepository,
        PgUserOrganizationRepository,
        PgUserRepository,
        PermissionChecker,
        PermissionChecker,
    >,
>;
pub type SharedProjectUc = Arc<
    ProjectUseCases<
        PgProjectRepository,
        PgUserProjectRepository,
        PgUserRepository,
        PermissionChecker,
    >,
>;
pub type SharedPipelineUc = Arc<
    PipelineUseCases<PgPipelineRepository, PgProjectRepository, PgJobRepository, PermissionChecker>,
>;
pub type SharedJobUc = Arc<JobUseCases<PgJobRepository, PermissionChecker>>;
pub type SharedJobLogUc = Arc<JobLogUseCases<PgJobLogRepository, PermissionChecker>>;
pub type SharedJobLogStreamUc =
    Arc<JobLogStreamUseCase<PgJobLogRepository, InMemoryJobLogStream, PermissionChecker>>;
pub type SharedAppUc =
    Arc<AppUseCases<PgAppRepository, Argon2HashService, PermissionChecker>>;
pub type SharedAppTokenUc =
    Arc<AppTokenUseCases<PgAppRepository, PgAppTokenRepository, Argon2HashService>>;
pub type SharedDispatchUc = Arc<DispatchUseCases<InMemoryWorkerRegistry, PermissionChecker>>;
pub type SharedWorkerUc = Arc<
    WorkerUseCases<
        PgAppRepository,
        PgWorkerRepository,
        Argon2HashService,
        PermissionChecker,
        PermissionChecker,
    >,
>;

// ── Services container ─────────────────────────────────────────────────

pub struct Services {
    pub db: PgPool,
    pub auth_uc: SharedAuthUc,
    #[cfg(feature = "signup")]
    pub signup_uc: SharedSignupUc,
    #[cfg(feature = "invitations")]
    pub invitation_uc: SharedInvitationUc,
    #[cfg(feature = "oauth-github")]
    pub oauth_uc: Option<SharedOAuthUc>,
    pub user_uc: SharedUserUc,
    pub org_uc: SharedOrgUc,
    pub project_uc: SharedProjectUc,
    pub pipeline_uc: SharedPipelineUc,
    pub job_uc: SharedJobUc,
    pub job_log_uc: SharedJobLogUc,
    pub job_log_stream_uc: SharedJobLogStreamUc,
    pub app_uc: SharedAppUc,
    pub app_token_uc: SharedAppTokenUc,
    pub worker_uc: SharedWorkerUc,
    pub worker_repo: Arc<PgWorkerRepository>,
    pub dispatch_uc: SharedDispatchUc,
    pub worker_registry: Arc<InMemoryWorkerRegistry>,
    pub job_log_stream: Arc<InMemoryJobLogStream>,
    pub grant_uc: SharedGrantUc,
    pub policy_uc: SharedPolicyUc,
    pub role_uc: SharedRoleUc,
    pub permission_checker: SharedPermissionChecker,
    pub session_repo: Arc<PgSessionRepository>,
    pub app_token_repo: Arc<PgAppTokenRepository>,
    pub user_role_repo: Arc<PgUserRoleRepository>,
    #[cfg(feature = "mail")]
    pub mailer: Arc<dyn Mailer>,
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
    let app_repo = Arc::new(PgAppRepository::new(db.clone()));
    let app_token_repo = Arc::new(PgAppTokenRepository::new(db.clone()));
    let worker_repo = Arc::new(PgWorkerRepository::new(db.clone()));
    let user_org_repo = Arc::new(PgUserOrganizationRepository::new(db.clone()));
    #[cfg(any(feature = "signup", feature = "oauth-github"))]
    let signup_repo = Arc::new(PgSignupRepository::new(db.clone()));
    #[cfg(feature = "invitations")]
    let invite_repo = Arc::new(PgInvitationRepository::new(db.clone()));
    let user_project_repo = Arc::new(PgUserProjectRepository::new(db.clone()));
    let user_role_repo = Arc::new(PgUserRoleRepository::new(db.clone()));
    let authz_provider = Arc::new(PgAuthzEntityProvider::new(db.clone()));
    let grant_repo = Arc::new(PgGrantRepository::new(db.clone()));
    let policy_repo = Arc::new(PgPolicyRepository::new(db.clone()));
    let hash_service = Arc::new(Argon2HashService::new());

    // Persistent audit trail; writes happen out-of-band on a background task.
    let audit_log: Arc<dyn AuditLog> = Arc::new(PgAuditLog::new(db.clone()));

    let permission_checker = Arc::new(
        CedarPermissionService::new(
            authz_provider,
            grant_repo.clone(),
            policy_repo.clone(),
            audit_log,
        )
        .await
        .map_err(|e| StartupError::Permission(e.to_string()))?,
    );

    let auth_uc = Arc::new(AuthUseCases::new(
        user_repo.clone(),
        session_repo.clone(),
        hash_service.clone(),
    ));
    #[cfg(feature = "signup")]
    let signup_uc = Arc::new(SignupUseCases::new(
        signup_repo.clone(),
        session_repo.clone(),
        hash_service.clone(),
        permission_checker.clone(),
    ));
    let user_uc = Arc::new(UserUseCases::new(
        user_repo.clone(),
        hash_service.clone(),
        permission_checker.clone(),
    ));
    let org_uc = Arc::new(OrganizationUseCases::new(
        org_repo.clone(),
        user_org_repo.clone(),
        user_repo.clone(),
        permission_checker.clone(),
        permission_checker.clone(),
    ));
    #[cfg(feature = "metering")]
    let project_uc = Arc::new(ProjectUseCases::new(
        project_repo.clone(),
        user_project_repo.clone(),
        user_repo.clone(),
        permission_checker.clone(),
        scylla_core::application::Quotas {
            max_projects_per_org: config.metering.max_projects_per_org,
        },
    ));
    #[cfg(not(feature = "metering"))]
    let project_uc = Arc::new(ProjectUseCases::new(
        project_repo.clone(),
        user_project_repo.clone(),
        user_repo.clone(),
        permission_checker.clone(),
    ));
    let pipeline_uc = Arc::new(PipelineUseCases::new(
        pipeline_repo.clone(),
        project_repo.clone(),
        job_repo.clone(),
        permission_checker.clone(),
    ));
    let job_uc = Arc::new(JobUseCases::new(job_repo.clone(), permission_checker.clone()));
    let job_log_uc = Arc::new(JobLogUseCases::new(
        job_log_repo.clone(),
        permission_checker.clone(),
    ));
    let app_uc = Arc::new(AppUseCases::new(
        app_repo.clone(),
        hash_service.clone(),
        permission_checker.clone(),
    ));
    let app_token_uc = Arc::new(AppTokenUseCases::new(
        app_repo.clone(),
        app_token_repo.clone(),
        hash_service.clone(),
    ));
    // Built here (before grant_uc) so revoking an app's grant can disconnect it.
    let worker_registry = Arc::new(InMemoryWorkerRegistry::new());
    let worker_uc = Arc::new(WorkerUseCases::new(
        app_repo.clone(),
        worker_repo.clone(),
        hash_service.clone(),
        permission_checker.clone(),
        permission_checker.clone(),
        worker_registry.clone(),
    ));
    let grant_uc = Arc::new(GrantUseCases::new(
        grant_repo.clone(),
        permission_checker.clone(),
        permission_checker.clone(),
        worker_registry.clone(),
    ));
    let policy_uc = Arc::new(PolicyUseCases::new(
        policy_repo.clone(),
        permission_checker.clone(),
        permission_checker.clone(),
    ));
    let role_uc = Arc::new(UserRoleUseCases::new(
        user_role_repo.clone(),
        permission_checker.clone(),
    ));

    if let Some(cfg) = &config.bootstrap {
        crate::bootstrap::bootstrap_admin(&user_uc, user_role_repo.as_ref(), cfg).await?;
    }

    // Mailer: real SMTP when configured, else a no-op (logs only).
    #[cfg(feature = "mail")]
    let mailer: Arc<dyn Mailer> = match &config.mail {
        Some(m) => Arc::new(
            LettreMailer::new(&m.host, m.port, m.username.clone(), m.password.clone(), &m.from)
                .map_err(|e| StartupError::Mail(e.to_string()))?,
        ),
        None => Arc::new(NoopMailer),
    };

    #[cfg(feature = "invitations")]
    let invitation_uc = Arc::new(InvitationUseCases::new(
        invite_repo.clone(),
        permission_checker.clone(),
        mailer.clone(),
        org_repo.clone(),
        user_repo.clone(),
        hash_service.clone(),
        session_repo.clone(),
        permission_checker.clone(),
    ));

    // GitHub OAuth: only wired when the app is configured with credentials.
    #[cfg(feature = "oauth-github")]
    let oauth_uc = match &config.oauth.github {
        Some(gh) => {
            let provider = GitHubOAuthProvider::new(
                gh.client_id.clone(),
                gh.client_secret.clone(),
                gh.redirect_uri.clone(),
            )
            .map_err(|e| StartupError::OAuth(e.to_string()))?;
            Some(Arc::new(OAuthUseCases::new(
                Arc::new(provider),
                Arc::new(PgOAuthIdentityRepository::new(db.clone())),
                signup_repo.clone(),
                user_repo.clone(),
                session_repo.clone(),
                hash_service.clone(),
                permission_checker.clone(),
            )))
        }
        None => None,
    };

    // In-process worker dispatch + job-log live-tail (mono-instance): jobs are
    // pushed to a connected worker's stream and log lines fan out through a
    // per-job broadcast. No message broker. (worker_registry is built above.)
    let job_log_stream = Arc::new(InMemoryJobLogStream::new());
    let job_log_stream_uc = Arc::new(JobLogStreamUseCase::new(
        job_log_repo.clone(),
        job_log_stream.clone(),
        permission_checker.clone(),
    ));
    let dispatch_uc = Arc::new(DispatchUseCases::new(
        worker_registry.clone(),
        permission_checker.clone(),
    ));

    Ok(Services {
        db,
        auth_uc,
        #[cfg(feature = "signup")]
        signup_uc,
        #[cfg(feature = "invitations")]
        invitation_uc,
        #[cfg(feature = "oauth-github")]
        oauth_uc,
        user_uc,
        org_uc,
        project_uc,
        pipeline_uc,
        job_uc,
        job_log_uc,
        job_log_stream_uc,
        app_uc,
        app_token_uc,
        worker_uc,
        worker_repo,
        dispatch_uc,
        worker_registry,
        job_log_stream,
        grant_uc,
        policy_uc,
        role_uc,
        permission_checker,
        session_repo,
        app_token_repo,
        user_role_repo,
        #[cfg(feature = "mail")]
        mailer,
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

// ── Graceful shutdown signal helper ────────────────────────────────────

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
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
pub async fn run_grpc<F>(
    config: &CoreConfig,
    services: &Services,
    shutdown: F,
) -> Result<(), StartupError>
where
    F: Future<Output = ()> + Send + 'static,
{
    use crate::grpc::{
        AppAuthHandler, AppHandler, AuthHandler, ConfigHandler, GrantHandler, JobHandler,
        OrganizationHandler, PipelineHandler, PolicyHandler, ProjectHandler, RoleHandler,
        UserHandler, WorkerAdminHandler, WorkerHandler, auth_interceptor::AuthInterceptor,
    };
    #[cfg(feature = "signup")]
    use crate::grpc::RegistrationHandler;
    #[cfg(feature = "invitations")]
    use crate::grpc::InvitationHandler;
    #[cfg(feature = "oauth-github")]
    use crate::grpc::OAuthHandler;
    use scylla_protocol::services::{
        app::app_auth_service_server::AppAuthServiceServer,
        app::app_service_server::AppServiceServer,
        auth::auth_service_server::AuthServiceServer,
        config::config_service_server::ConfigServiceServer,
        job::job_service_server::JobServiceServer,
        organization::organization_service_server::OrganizationServiceServer,
        permission::grant_service_server::GrantServiceServer,
        permission::policy_service_server::PolicyServiceServer,
        permission::role_service_server::RoleServiceServer,
        pipeline::pipeline_service_server::PipelineServiceServer,
        project::project_service_server::ProjectServiceServer,
        user::user_service_server::UserServiceServer,
        worker::worker_service_server::WorkerServiceServer,
        worker_admin::worker_admin_service_server::WorkerAdminServiceServer,
    };
    #[cfg(feature = "signup")]
    use scylla_protocol::services::registration::registration_service_server::RegistrationServiceServer;
    #[cfg(feature = "invitations")]
    use scylla_protocol::services::invitation::{
        invitation_accept_service_server::InvitationAcceptServiceServer,
        invitation_service_server::InvitationServiceServer,
    };
    #[cfg(feature = "oauth-github")]
    use scylla_protocol::services::oauth::oauth_service_server::OauthServiceServer;
    use tonic::transport::Server;
    use tonic_async_interceptor::async_interceptor;
    use tonic_web::GrpcWebLayer;
    use tower::ServiceBuilder;
    use tower_http::trace::TraceLayer;

    let auth_handler = AuthHandler::new(services.auth_uc.clone());
    let user_handler = UserHandler::new(services.user_uc.clone());
    let org_handler = OrganizationHandler::new(services.org_uc.clone());
    let project_handler = ProjectHandler::new(services.project_uc.clone());
    let pipeline_handler = PipelineHandler::new(
        services.pipeline_uc.clone(),
        services.dispatch_uc.clone(),
    );
    let job_handler = JobHandler::new(
        services.job_uc.clone(),
        services.job_log_uc.clone(),
        services.job_log_stream_uc.clone(),
    );
    let app_handler = AppHandler::new(services.app_uc.clone());
    let app_auth_handler = AppAuthHandler::new(services.app_token_uc.clone());
    let worker_handler = WorkerHandler::new(
        services.worker_registry.clone(),
        services.job_log_stream.clone(),
        services.job_uc.clone(),
        services.job_log_uc.clone(),
        services.worker_repo.clone(),
    );
    let worker_admin_handler = WorkerAdminHandler::new(services.worker_uc.clone());
    let policy_handler = PolicyHandler::new(services.policy_uc.clone());
    let grant_handler = GrantHandler::new(services.grant_uc.clone());
    let role_handler = RoleHandler::new(services.role_uc.clone());
    #[cfg(feature = "invitations")]
    let invitation_handler = InvitationHandler::new(services.invitation_uc.clone());

    let auth_interceptor = async_interceptor(AuthInterceptor::new(
        services.session_repo.clone(),
        services.app_token_repo.clone(),
    ));
    let cors_layer = build_cors_layer(&config.cors);

    tracing::info!("gRPC server listening on {}", config.grpc.address);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(scylla_protocol::services::FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .map_err(|e| StartupError::Reflection(e.to_string()))?;

    let auth_service = AuthServiceServer::new(auth_handler);

    // Public capability discovery — no auth interceptor, like auth/login.
    let config_service = ConfigServiceServer::new(ConfigHandler);

    // Public app credential exchange — the secret is the credential, so no
    // interceptor; it mints the bearer token apps use on every other call.
    let app_auth_service = AppAuthServiceServer::new(app_auth_handler);

    // Public self-service signup — only present in SaaS (`signup` feature) builds.
    #[cfg(feature = "signup")]
    let registration_service =
        RegistrationServiceServer::new(RegistrationHandler::new(services.signup_uc.clone()));

    // Public invitation acceptance — the token is the credential, so no interceptor.
    #[cfg(feature = "invitations")]
    let invitation_accept_service =
        InvitationAcceptServiceServer::new(invitation_handler.clone());

    // Public GitHub OAuth — present only when configured with credentials.
    #[cfg(feature = "oauth-github")]
    let oauth_service = services
        .oauth_uc
        .as_ref()
        .map(|uc| OauthServiceServer::new(OAuthHandler::new(uc.clone())));

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

    let app_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(AppServiceServer::new(app_handler));

    // Authenticated worker stream (app token). Presence = the open stream.
    let worker_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(WorkerServiceServer::new(worker_handler));

    // Authenticated worker management + introspection (dashboard).
    let worker_admin_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(WorkerAdminServiceServer::new(worker_admin_handler));

    let policy_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(PolicyServiceServer::new(policy_handler));

    let grant_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(GrantServiceServer::new(grant_handler));

    // Authenticated invitation management (org-admins).
    #[cfg(feature = "invitations")]
    let invitation_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(InvitationServiceServer::new(invitation_handler));

    let role_service = ServiceBuilder::new()
        .layer(auth_interceptor)
        .service(RoleServiceServer::new(role_handler));

    let router = Server::builder()
        .accept_http1(true)
        .layer(TraceLayer::new_for_grpc())
        .layer(cors_layer)
        .layer(GrpcWebLayer::new())
        .add_service(reflection)
        .add_service(auth_service)
        .add_service(config_service)
        .add_service(app_auth_service)
        .add_service(user_service)
        .add_service(org_service)
        .add_service(project_service)
        .add_service(pipeline_service)
        .add_service(job_service)
        .add_service(app_service)
        .add_service(worker_service)
        .add_service(worker_admin_service)
        .add_service(policy_service)
        .add_service(grant_service)
        .add_service(role_service);

    // SaaS-only services, registered behind their cargo feature.
    #[cfg(feature = "signup")]
    let router = router.add_service(registration_service);
    #[cfg(feature = "invitations")]
    let router = router
        .add_service(invitation_service)
        .add_service(invitation_accept_service);
    #[cfg(feature = "oauth-github")]
    let router = match oauth_service {
        Some(svc) => router.add_service(svc),
        None => router,
    };

    router.serve_with_shutdown(config.grpc.address, shutdown).await?;

    Ok(())
}
