pub mod bootstrap;
pub mod config;
pub mod error;
pub mod startup;
pub mod rest;

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "grpc")]
pub use grpc::{
    AuthContext, AuthHandler, JobHandler, OrganizationHandler, PipelineHandler, ProjectHandler,
    UserHandler, auth_interceptor, domain_error_to_status, domain_to_proto_metadata, job_to_proto,
    middleware::extract_auth_context, organization_to_proto, pipeline_to_proto, project_to_proto,
    proto_to_domain_pagination, user_to_proto,
};

#[cfg(feature = "grpc")]
pub use config::GrpcConfig;
pub use config::{BootstrapConfig, CoreConfig, CorsConfig};
pub use error::{BootstrapError, ConfigError, StartupError};
pub use startup::{
    Services, SharedAuthUc, SharedGrantUc, SharedJobLogStreamUc, SharedJobLogUc, SharedJobUc,
    SharedOrgUc, SharedPipelineUc, SharedProjectUc, SharedUserUc, SharedWebhookIngressUc,
    build_cors_layer, init_services, run_grpc, run_webhook, shutdown_signal,
};
