pub mod bootstrap;
pub mod config;
pub mod error;
pub mod startup;

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "grpc")]
pub use grpc::{
    AuthContext, AuthHandler, JobHandler, OrganizationHandler, PipelineHandler, ProjectHandler,
    UserHandler, auth_interceptor, domain_error_to_status, domain_to_proto_metadata, job_to_proto,
    middleware::extract_auth_context, organization_to_proto, pipeline_to_proto, project_to_proto,
    proto_to_domain_pagination, user_to_proto,
};

pub use config::{BootstrapConfig, CoreConfig, CorsConfig};
#[cfg(feature = "grpc")]
pub use config::GrpcConfig;
pub use error::{BootstrapError, ConfigError, StartupError};
pub use startup::{
    SharedAuthUc, SharedGrantUc, SharedJobLogStreamUc, SharedJobLogUc, SharedJobUc, SharedOrgUc,
    SharedPipelineUc, SharedProjectUc, SharedUserUc, Services, build_cors_layer, init_services,
    run_grpc, shutdown_signal,
};
