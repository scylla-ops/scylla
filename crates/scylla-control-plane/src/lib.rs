//! The Scylla control plane: use cases, adapters, the gRPC and HTTP surfaces,
//! and the binary that composes them.
//!
//! The domain model itself lives in the `scylla-core` kernel, which the agents
//! also depend on. It is re-exported below so that `crate::domain::...` keeps
//! naming it from anywhere in this crate.

/// The domain model, re-exported from the [`scylla_core`] kernel.
///
/// This is what lets every `crate::domain::...` path in this crate keep
/// resolving now that the model lives in its own crate. Extracting the kernel
/// would otherwise have meant rewriting 484 import sites for no behavioural
/// gain, and buried the actual change under the churn.
pub use scylla_core::domain;

pub mod application;
pub mod infrastructure;

pub mod bootstrap;
pub mod config;
pub mod error;
pub mod rest;
pub mod runtime;
pub mod startup;

pub mod grpc;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_support;

pub use grpc::{
    AuthContext, AuthHandler, JobHandler, OrganizationHandler, PipelineHandler, ProjectHandler,
    UserHandler, auth_interceptor, domain_error_to_status, domain_to_proto_metadata, job_to_proto,
    middleware::extract_auth_context, organization_to_proto, pipeline_to_proto, project_to_proto,
    proto_to_domain_pagination, user_to_proto,
};

pub use config::GrpcConfig;
pub use config::{BootstrapConfig, ControlPlaneConfig, CorsConfig};
pub use error::{BootstrapError, ConfigError, StartupError};
pub use startup::{
    Services, SharedAuthUc, SharedGrantUc, SharedJobLogStreamUc, SharedJobLogUc, SharedJobUc,
    SharedOrgUc, SharedPipelineUc, SharedProjectUc, SharedUserUc, SharedWebhookIngressUc,
    build_cors_layer, init_services, run_grpc, run_webhook, shutdown_signal,
};
