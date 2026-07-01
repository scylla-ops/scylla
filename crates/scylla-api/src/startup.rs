use crate::config::CoreConfig;
use crate::error::StartupError;
use http::{HeaderName, HeaderValue, Method};
use tokio::sync::Notify;
use scylla_core::application::{
    AgentDispatch, AgentUseCases, AppTokenUseCases, AppUseCases, AuditLog, AuthUseCases,
    BootstrapUseCases, CronSchedule, DispatchSecretResolver, DispatchUseCases, GrantUseCases,
    InvitationUseCases, JobLogStreamUseCase, JobLogUseCases, JobReaper, JobUseCases, Mailer,
    NoopMailer, OAuthUseCases, OrganizationUseCases, PendingJobScheduler, PipelineUseCases,
    PolicyUseCases, ProjectUseCases, RoleUseCases, SecretCipher, SecretResolver, SecretUseCases,
    SignupUseCases, TriggerCronScheduler, TriggerFireUseCases, TriggerFiring, TriggerUseCases,
    UserUseCases, WebhookIngressUseCases,
};
use scylla_core::infrastructure::LettreMailer;
use scylla_core::infrastructure::{
    Argon2HashService, CedarPermissionService, ChaChaSecretCipher, CronScheduleService,
    GitHubOAuthProvider,
    InMemoryAgentRegistry, InMemoryJobLogStream, PgAgentRepository, PgAppCredentialRepository,
    PgAppRepository, PgAppTokenRepository, PgAuditLog, PgAuthzEntityProvider,
    PgDefaultRoleBindingRepository, PgGrantRepository, PgInvitationRepository, PgJobLogRepository,
    PgJobRepository, PgOAuthIdentityRepository, PgOrganizationRepository, PgPipelineRepository,
    PgPolicyRepository, PgProjectRepository, PgRoleRepository, PgSecretRepository,
    PgSessionRepository, PgSignupRepository, PgTriggerDeliveryRepository, PgTriggerRepository,
    PgUserOrganizationRepository, PgUserProjectRepository, PgUserRepository,
};
use sqlx::PgPool;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

// ── Concrete type aliases ──────────────────────────────────────────────

pub type PermissionChecker = CedarPermissionService<PgAuthzEntityProvider>;
pub type SharedPermissionChecker = Arc<PermissionChecker>;
pub type SharedGrantUc =
    Arc<GrantUseCases<PgGrantRepository, PermissionChecker, PermissionChecker>>;
pub type SharedRoleUc =
    Arc<RoleUseCases<PgRoleRepository, PgGrantRepository, PermissionChecker, PermissionChecker>>;
pub type SharedPolicyUc =
    Arc<PolicyUseCases<PgPolicyRepository, PermissionChecker, PermissionChecker>>;

pub type SharedAuthUc = Arc<AuthUseCases<PgUserRepository, PgSessionRepository, Argon2HashService>>;
pub type SharedSignupUc = Arc<
    SignupUseCases<PgSignupRepository, PgSessionRepository, Argon2HashService, PermissionChecker>,
>;
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
        PermissionChecker,
    >,
>;
pub type SharedPipelineUc = Arc<
    PipelineUseCases<PgPipelineRepository, PgProjectRepository, PgJobRepository, PermissionChecker>,
>;
pub type SharedJobUc = Arc<JobUseCases<PgJobRepository, PermissionChecker>>;
pub type SharedSecretUc = Arc<SecretUseCases<PgSecretRepository, PermissionChecker>>;
pub type SharedJobLogUc = Arc<JobLogUseCases<PgJobLogRepository, PermissionChecker>>;
pub type SharedJobLogStreamUc =
    Arc<JobLogStreamUseCase<PgJobLogRepository, InMemoryJobLogStream, PermissionChecker>>;
pub type SharedAppUc = Arc<
    AppUseCases<PgAppRepository, PgAppCredentialRepository, Argon2HashService, PermissionChecker>,
>;
pub type SharedAppTokenUc = Arc<
    AppTokenUseCases<
        PgAppRepository,
        PgAppTokenRepository,
        PgAppCredentialRepository,
        Argon2HashService,
    >,
>;
pub type SharedDispatchUc = Arc<DispatchUseCases<InMemoryAgentRegistry, PermissionChecker>>;
pub type SharedAgentUc = Arc<
    AgentUseCases<
        PgAppRepository,
        PgAgentRepository,
        Argon2HashService,
        PermissionChecker,
        PermissionChecker,
    >,
>;
pub type SharedTriggerUc = Arc<
    TriggerUseCases<
        PgTriggerRepository,
        PgPipelineRepository,
        PgProjectRepository,
        PgAppRepository,
        Argon2HashService,
        PermissionChecker,
        PermissionChecker,
    >,
>;
pub type SharedTriggerFireUc = Arc<
    TriggerFireUseCases<
        PgTriggerRepository,
        PgPipelineRepository,
        PgProjectRepository,
        PgAppRepository,
        PgJobRepository,
        PermissionChecker,
        InMemoryAgentRegistry,
    >,
>;
pub type SharedWebhookIngressUc =
    Arc<WebhookIngressUseCases<PgTriggerRepository, PgTriggerDeliveryRepository>>;

// ── Services container ─────────────────────────────────────────────────

pub struct Services {
    pub db: PgPool,
    pub auth_uc: SharedAuthUc,
    pub signup_uc: SharedSignupUc,
    pub invitation_uc: SharedInvitationUc,
    pub oauth_uc: Option<SharedOAuthUc>,
    pub user_uc: SharedUserUc,
    pub org_uc: SharedOrgUc,
    pub project_uc: SharedProjectUc,
    pub pipeline_uc: SharedPipelineUc,
    pub trigger_uc: SharedTriggerUc,
    pub trigger_fire_uc: SharedTriggerFireUc,
    pub webhook_ingress_uc: SharedWebhookIngressUc,
    pub secret_uc: SharedSecretUc,
    pub job_uc: SharedJobUc,
    pub job_log_uc: SharedJobLogUc,
    pub job_log_stream_uc: SharedJobLogStreamUc,
    pub app_uc: SharedAppUc,
    pub app_token_uc: SharedAppTokenUc,
    pub agent_uc: SharedAgentUc,
    pub agent_repo: Arc<PgAgentRepository>,
    pub dispatch_uc: SharedDispatchUc,
    pub agent_registry: Arc<InMemoryAgentRegistry>,
    /// Poked by the agent handler on connect to wake the pending-job scheduler.
    pub pending_signal: Arc<Notify>,
    pub job_log_stream: Arc<InMemoryJobLogStream>,
    pub grant_uc: SharedGrantUc,
    pub role_uc: SharedRoleUc,
    pub policy_uc: SharedPolicyUc,
    pub permission_checker: SharedPermissionChecker,
    pub session_repo: Arc<PgSessionRepository>,
    pub app_token_repo: Arc<PgAppTokenRepository>,
    pub mailer: Arc<dyn Mailer>,
}

pub async fn init_services(config: &CoreConfig) -> Result<Services, StartupError> {
    let db = scylla_core::infrastructure::init_db(&config.database).await?;

    let user_repo = Arc::new(PgUserRepository::new(db.clone()));
    let session_repo = Arc::new(PgSessionRepository::new(db.clone()));
    let org_repo = Arc::new(PgOrganizationRepository::new(db.clone()));
    let project_repo = Arc::new(PgProjectRepository::new(db.clone()));
    let pipeline_repo = Arc::new(PgPipelineRepository::new(db.clone()));
    let secret_repo = Arc::new(PgSecretRepository::new(db.clone()));
    let job_repo = Arc::new(PgJobRepository::new(db.clone()));
    let job_log_repo = Arc::new(PgJobLogRepository::new(db.clone()));
    let app_repo = Arc::new(PgAppRepository::new(db.clone()));
    let app_credential_repo = Arc::new(PgAppCredentialRepository::new(db.clone()));
    let app_token_repo = Arc::new(PgAppTokenRepository::new(db.clone()));
    let agent_repo = Arc::new(PgAgentRepository::new(db.clone()));
    let user_org_repo = Arc::new(PgUserOrganizationRepository::new(db.clone()));
    let signup_repo = Arc::new(PgSignupRepository::new(db.clone()));
    let invite_repo = Arc::new(PgInvitationRepository::new(db.clone()));
    let user_project_repo = Arc::new(PgUserProjectRepository::new(db.clone()));
    let authz_provider = Arc::new(PgAuthzEntityProvider::new(db.clone()));
    let role_repo = Arc::new(PgRoleRepository::new(db.clone()));
    let default_role_repo = Arc::new(PgDefaultRoleBindingRepository::new(db.clone()));
    let grant_repo = Arc::new(PgGrantRepository::new(db.clone()));
    let policy_repo = Arc::new(PgPolicyRepository::new(db.clone()));
    let hash_service = Arc::new(Argon2HashService::new());

    // Project-secret encryption. Disabled (errors on use) when no master key is
    // configured; the rest of the server still boots.
    let secret_cipher: Arc<dyn SecretCipher> = Arc::new(ChaChaSecretCipher::from_hex_key(
        config.secrets.as_ref().map(|s| s.master_key.as_str()),
    )?);
    let secret_resolver: Arc<dyn SecretResolver> = Arc::new(DispatchSecretResolver::new(
        secret_repo.clone(),
        secret_cipher.clone(),
    ));

    // Persistent audit trail; writes happen out-of-band on a background task.
    let audit_log: Arc<dyn AuditLog> = Arc::new(PgAuditLog::new(db.clone()));

    let permission_checker = Arc::new(
        CedarPermissionService::new(
            authz_provider,
            role_repo.clone(),
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
    let signup_uc = Arc::new(SignupUseCases::new(
        signup_repo.clone(),
        session_repo.clone(),
        hash_service.clone(),
        permission_checker.clone(),
        default_role_repo.clone(),
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
        default_role_repo.clone(),
    ));
    let project_uc = Arc::new(ProjectUseCases::new(
        project_repo.clone(),
        user_project_repo.clone(),
        user_repo.clone(),
        permission_checker.clone(),
        permission_checker.clone(),
        default_role_repo.clone(),
        scylla_core::application::Quotas {
            max_projects_per_org: config.metering.max_projects_per_org,
        },
    ));
    let secret_uc = Arc::new(SecretUseCases::new(
        secret_repo.clone(),
        secret_cipher.clone(),
        permission_checker.clone(),
    ));
    let pipeline_uc = Arc::new(PipelineUseCases::new(
        pipeline_repo.clone(),
        project_repo.clone(),
        job_repo.clone(),
        permission_checker.clone(),
        secret_resolver.clone(),
    ));
    let job_uc = Arc::new(JobUseCases::new(
        job_repo.clone(),
        permission_checker.clone(),
    ));
    let job_log_uc = Arc::new(JobLogUseCases::new(
        job_log_repo.clone(),
        permission_checker.clone(),
    ));
    // Built before app_uc/grant_uc so disabling, deleting, or revoking an app's
    // grant can drop its live agent stream immediately.
    let agent_registry = Arc::new(InMemoryAgentRegistry::new());
    let app_uc = Arc::new(AppUseCases::new(
        app_repo.clone(),
        app_credential_repo.clone(),
        hash_service.clone(),
        permission_checker.clone(),
        agent_registry.clone(),
    ));
    let app_token_uc = Arc::new(AppTokenUseCases::new(
        app_repo.clone(),
        app_token_repo.clone(),
        app_credential_repo.clone(),
        hash_service.clone(),
    ));
    let agent_uc = Arc::new(AgentUseCases::new(
        app_repo.clone(),
        agent_repo.clone(),
        hash_service.clone(),
        permission_checker.clone(),
        permission_checker.clone(),
        agent_registry.clone(),
    ));
    let grant_uc = Arc::new(GrantUseCases::new(
        grant_repo.clone(),
        role_repo.clone(),
        permission_checker.clone(),
        permission_checker.clone(),
        agent_registry.clone(),
    ));
    let role_uc = Arc::new(RoleUseCases::new(
        role_repo.clone(),
        grant_repo.clone(),
        permission_checker.clone(),
        permission_checker.clone(),
    ));
    let policy_uc = Arc::new(PolicyUseCases::new(
        policy_repo.clone(),
        permission_checker.clone(),
        permission_checker.clone(),
    ));
    if let Some(cfg) = &config.bootstrap {
        // Bootstrap mints a System-scoped `system-admin` grant via the grant use
        // case (replaces the former global-role assignment).
        let bootstrap_uc = BootstrapUseCases::new(user_uc.clone(), grant_uc.clone());
        crate::bootstrap::bootstrap_admin(&bootstrap_uc, cfg).await?;
    }

    // Mailer: real SMTP when configured, else a no-op (logs only).
    let mailer: Arc<dyn Mailer> = match &config.mail {
        Some(m) => Arc::new(
            LettreMailer::new(
                &m.host,
                m.port,
                m.username.clone(),
                m.password.clone(),
                &m.from,
            )
            .map_err(|e| StartupError::Mail(e.to_string()))?,
        ),
        None => Arc::new(NoopMailer),
    };

    let invitation_uc = Arc::new(InvitationUseCases::new(
        invite_repo.clone(),
        permission_checker.clone(),
        mailer.clone(),
        org_repo.clone(),
        user_repo.clone(),
        hash_service.clone(),
        session_repo.clone(),
        permission_checker.clone(),
        role_repo.clone(),
    ));

    // GitHub OAuth: only wired when the app is configured with credentials.
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
                default_role_repo.clone(),
            )))
        }
        None => None,
    };

    // In-process agent dispatch + job-log live-tail (mono-instance): jobs are
    // pushed to a connected agent's stream and log lines fan out through a
    // per-job broadcast. No message broker. (agent_registry is built above.)
    let job_log_stream = Arc::new(InMemoryJobLogStream::new());
    let job_log_stream_uc = Arc::new(JobLogStreamUseCase::new(
        job_log_repo.clone(),
        job_log_stream.clone(),
        permission_checker.clone(),
    ));
    let dispatch_uc = Arc::new(DispatchUseCases::new(
        agent_registry.clone(),
        permission_checker.clone(),
    ));

    let trigger_repo = Arc::new(PgTriggerRepository::new(db.clone()));
    let trigger_delivery_repo = Arc::new(PgTriggerDeliveryRepository::new(db.clone()));
    // One cron-schedule service, shared by the trigger CRUD (anchors next_fire_at
    // on create/update/re-enable) and the firing scheduler (seed + claim).
    let cron_schedule: Arc<dyn CronSchedule> = Arc::new(CronScheduleService::new());
    let trigger_uc = Arc::new(TriggerUseCases::new(
        trigger_repo.clone(),
        pipeline_repo.clone(),
        project_repo.clone(),
        app_repo.clone(),
        hash_service.clone(),
        permission_checker.clone(),
        permission_checker.clone(),
        secret_cipher.clone(),
        cron_schedule.clone(),
    ));
    let trigger_fire_uc = Arc::new(TriggerFireUseCases::new(
        trigger_repo.clone(),
        pipeline_repo.clone(),
        project_repo.clone(),
        app_repo.clone(),
        pipeline_uc.clone(),
        dispatch_uc.clone(),
    ));
    // Webhook ingress: shares the trigger-runner fire path; firing is the
    // TriggerFiring trait object the cron scheduler also uses.
    let webhook_ingress_uc = Arc::new(WebhookIngressUseCases::new(
        trigger_repo.clone(),
        trigger_delivery_repo.clone(),
        secret_cipher.clone(),
        trigger_fire_uc.clone() as Arc<dyn TriggerFiring>,
    ));

    // Cron firing engine: each tick seeds first-occurrences for new cron triggers
    // and fires the due ones (each as the org's trigger-runner App, through the
    // one RunPipeline check). Like the pending-job scheduler it is detached for
    // the process lifetime; the first interval tick fires immediately, so a
    // pre-restart backlog is picked up at boot. A 15s tick bounds firing latency
    // to well under cron's one-minute granularity without busy-polling.
    {
        let firing: Arc<dyn TriggerFiring> = trigger_fire_uc.clone();
        let scheduler =
            TriggerCronScheduler::new(trigger_repo.clone(), firing, cron_schedule.clone());
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(15));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tick.tick().await;
                scheduler.tick().await;
            }
        });
    }

    // Pending-job scheduler: places jobs that were minted while no worker was
    // connected. Woken by the agent handler on connect (`pending_signal`), on a
    // periodic tick as a safety net, and once at boot to pick up a pre-restart
    // backlog. Runs for the process lifetime.
    let pending_signal = Arc::new(Notify::new());
    {
        let scheduler = PendingJobScheduler::new(
            job_repo.clone(),
            pipeline_repo.clone(),
            dispatch_uc.clone(),
            secret_resolver.clone(),
        );
        let signal = pending_signal.clone();
        tokio::spawn(async move {
            scheduler.drain().await;
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(30));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    () = signal.notified() => {}
                    _ = tick.tick() => {}
                }
                scheduler.drain().await;
            }
        });
    }

    // Orphaned-job reaper: a job is `running` only while its agent owns it. If
    // that agent's stream drops without a terminal report (crash, network, or a
    // control plane restart that forgets every live stream), the job would sit
    // `running` forever. Level-triggered reconciliation orphans every running
    // job whose agent is not currently connected: once at boot (nothing is
    // connected yet, so every pre-restart running job is cleared) and on a
    // periodic tick.
    {
        let reaper = JobReaper::new(job_repo.clone());
        let registry = agent_registry.clone();
        tokio::spawn(async move {
            reaper.reap(&[]).await;
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(30));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tick.tick().await;
                reaper.reap(&registry.connected()).await;
            }
        });
    }

    Ok(Services {
        db,
        auth_uc,
        signup_uc,
        invitation_uc,
        oauth_uc,
        user_uc,
        org_uc,
        project_uc,
        pipeline_uc,
        trigger_uc,
        trigger_fire_uc,
        webhook_ingress_uc,
        secret_uc,
        job_uc,
        job_log_uc,
        job_log_stream_uc,
        app_uc,
        app_token_uc,
        agent_uc,
        agent_repo,
        dispatch_uc,
        agent_registry,
        pending_signal,
        job_log_stream,
        grant_uc,
        role_uc,
        policy_uc,
        permission_checker,
        session_repo,
        app_token_repo,
        mailer,
    })
}

// ── CORS builder ───────────────────────────────────────────────────────

pub fn build_cors_layer(cors: &crate::config::CorsConfig) -> CorsLayer {
    let mut layer = CorsLayer::new();

    if cors.allow_origins.iter().any(|o| o == "*") {
        // Wildcard origin reflected alongside the `authorization` header is a
        // permissive posture for a token-authenticated API. Fine for local dev,
        // dangerous in production — make the choice loud rather than silent.
        tracing::warn!(
            "CORS allow_origins contains '*': any origin is accepted. Do NOT use this in production — set explicit origins in config."
        );
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
        AgentAdminHandler, AgentHandler, AppAuthHandler, AppHandler, AuthHandler, GrantHandler,
        InvitationHandler, JobHandler, OAuthHandler, OrganizationHandler, PipelineHandler,
        PolicyHandler, ProjectHandler, RegistrationHandler, RoleHandler, SecretHandler,
        TriggerHandler, UserHandler, auth_interceptor::AuthInterceptor,
    };
    use scylla_protocol::services::invitation::{
        invitation_accept_service_server::InvitationAcceptServiceServer,
        invitation_service_server::InvitationServiceServer,
    };
    use scylla_protocol::services::oauth::oauth_service_server::OauthServiceServer;
    use scylla_protocol::services::registration::registration_service_server::RegistrationServiceServer;
    use scylla_protocol::services::{
        agent::agent_service_server::AgentServiceServer,
        agent_admin::agent_admin_service_server::AgentAdminServiceServer,
        app::app_auth_service_server::AppAuthServiceServer,
        app::app_service_server::AppServiceServer, auth::auth_service_server::AuthServiceServer,
        job::job_service_server::JobServiceServer,
        organization::organization_service_server::OrganizationServiceServer,
        permission::grant_service_server::GrantServiceServer,
        permission::policy_service_server::PolicyServiceServer,
        permission::role_service_server::RoleServiceServer,
        pipeline::pipeline_service_server::PipelineServiceServer,
        project::project_service_server::ProjectServiceServer,
        secret::secret_service_server::SecretServiceServer,
        trigger::trigger_service_server::TriggerServiceServer,
        user::user_service_server::UserServiceServer,
    };
    use tonic::transport::Server;
    use tonic_async_interceptor::async_interceptor;
    use tonic_web::GrpcWebLayer;
    use tower::ServiceBuilder;
    use tower_http::trace::TraceLayer;

    let auth_handler = AuthHandler::new(services.auth_uc.clone());
    let user_handler = UserHandler::new(services.user_uc.clone());
    let org_handler = OrganizationHandler::new(services.org_uc.clone());
    let project_handler = ProjectHandler::new(services.project_uc.clone());
    let pipeline_handler =
        PipelineHandler::new(services.pipeline_uc.clone(), services.dispatch_uc.clone());
    let trigger_handler = TriggerHandler::new(
        services.trigger_uc.clone(),
        services.trigger_fire_uc.clone(),
        config
            .webhook
            .as_ref()
            .and_then(|w| w.public_base_url.clone()),
    );
    let job_handler = JobHandler::new(
        services.job_uc.clone(),
        services.job_log_uc.clone(),
        services.job_log_stream_uc.clone(),
    );
    let app_handler = AppHandler::new(services.app_uc.clone());
    let secret_handler = SecretHandler::new(services.secret_uc.clone());
    let app_auth_handler = AppAuthHandler::new(services.app_token_uc.clone());
    let agent_handler = AgentHandler::new(
        services.agent_registry.clone(),
        services.job_log_stream.clone(),
        services.job_uc.clone(),
        services.job_log_uc.clone(),
        services.agent_repo.clone(),
        services.pending_signal.clone(),
    );
    let agent_admin_handler = AgentAdminHandler::new(services.agent_uc.clone());
    let policy_handler = PolicyHandler::new(services.policy_uc.clone());
    let grant_handler = GrantHandler::new(services.grant_uc.clone());
    let role_handler = RoleHandler::new(services.role_uc.clone());
    let invitation_handler = InvitationHandler::new(services.invitation_uc.clone());

    let auth_interceptor = async_interceptor(AuthInterceptor::new(
        services.session_repo.clone(),
        services.app_token_repo.clone(),
    ));
    let cors_layer = build_cors_layer(&config.cors);

    tracing::info!("gRPC server listening on {}", config.grpc.address);

    // Expose BOTH reflection variants so any client works: v1 (current spec) and
    // v1alpha (older clients — grpcurl defaults, some MCP/reflection bridges).
    // They are distinct gRPC services (grpc.reflection.v1[alpha].ServerReflection),
    // so they coexist on the same server without a route clash.
    let reflection_v1 = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(scylla_protocol::services::FILE_DESCRIPTOR_SET)
        .build_v1()
        .map_err(|e| StartupError::Reflection(e.to_string()))?;
    let reflection_v1alpha = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(scylla_protocol::services::FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .map_err(|e| StartupError::Reflection(e.to_string()))?;

    let auth_service = AuthServiceServer::new(auth_handler);

    // Public app credential exchange — the secret is the credential, so no
    // interceptor; it mints the bearer token apps use on every other call.
    let app_auth_service = AppAuthServiceServer::new(app_auth_handler);

    // Public self-service signup.
    let registration_service =
        RegistrationServiceServer::new(RegistrationHandler::new(services.signup_uc.clone()));

    // Public invitation acceptance — the token is the credential, so no interceptor.
    let invitation_accept_service = InvitationAcceptServiceServer::new(invitation_handler.clone());

    // Public GitHub OAuth — present only when configured with credentials.
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

    let trigger_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(TriggerServiceServer::new(trigger_handler));

    let job_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(JobServiceServer::new(job_handler));

    let app_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(AppServiceServer::new(app_handler));

    let secret_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(SecretServiceServer::new(secret_handler));

    // Authenticated agent stream (app token). Presence = the open stream.
    let agent_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(AgentServiceServer::new(agent_handler));

    // Authenticated agent management + introspection (dashboard).
    let agent_admin_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(AgentAdminServiceServer::new(agent_admin_handler));

    let policy_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(PolicyServiceServer::new(policy_handler));

    let grant_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(GrantServiceServer::new(grant_handler));

    let role_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(RoleServiceServer::new(role_handler));

    // Authenticated invitation management (org-admins).
    let invitation_service = ServiceBuilder::new()
        .layer(auth_interceptor.clone())
        .service(InvitationServiceServer::new(invitation_handler));

    let router = Server::builder()
        .accept_http1(true)
        .layer(TraceLayer::new_for_grpc())
        .layer(cors_layer)
        .layer(GrpcWebLayer::new())
        .add_service(reflection_v1)
        .add_service(reflection_v1alpha)
        .add_service(auth_service)
        .add_service(app_auth_service)
        .add_service(user_service)
        .add_service(org_service)
        .add_service(project_service)
        .add_service(pipeline_service)
        .add_service(trigger_service)
        .add_service(secret_service)
        .add_service(job_service)
        .add_service(app_service)
        .add_service(agent_service)
        .add_service(agent_admin_service)
        .add_service(policy_service)
        .add_service(grant_service)
        .add_service(role_service)
        .add_service(registration_service)
        .add_service(invitation_service)
        .add_service(invitation_accept_service);

    // GitHub OAuth is only registered when configured with credentials.
    let router = match oauth_service {
        Some(svc) => router.add_service(svc),
        None => router,
    };

    router
        .serve_with_shutdown(config.grpc.address, shutdown)
        .await?;

    Ok(())
}

// ── Webhook ingress HTTP server ────────────────────────────────────────────

/// Serve the inbound-webhook HTTP API on its own port until `shutdown` resolves.
/// Separate from the gRPC server: a public, unauthenticated-at-the-edge surface
/// (requests are authenticated per-trigger by HMAC inside the handler).
pub async fn run_webhook<F>(
    address: SocketAddr,
    ingress: SharedWebhookIngressUc,
    shutdown: F,
) -> Result<(), StartupError>
where
    F: Future<Output = ()> + Send + 'static,
{
    let app = crate::rest::webhook::router(ingress);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|e| StartupError::Webhook(format!("bind {address}: {e}")))?;
    tracing::info!("webhook ingress listening on {address}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(|e| StartupError::Webhook(e.to_string()))?;
    Ok(())
}
