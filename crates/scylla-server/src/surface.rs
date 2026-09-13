//! What an edition puts on the listener next to the core's own services.
//!
//! Every method is generic over what is contributed (a service by its
//! generated server type, routes as a router), so a new service in an edition
//! never changes this crate.

use std::convert::Infallible;
use tonic::body::Body;
use tonic::server::NamedService;
use tonic::service::RoutesBuilder;
use tower::{Layer, Service};

/// The bearer-token interceptor every authenticated service sits behind, as
/// the layer type `tower::ServiceBuilder::layer` takes.
pub(crate) type Auth = tonic_async_interceptor::AsyncInterceptorLayer<
    scylla_core::grpc::auth_interceptor::AuthInterceptor<
        scylla_db::PgSessionRepository,
        scylla_db::PgAppTokenRepository,
    >,
>;

/// Adds one gRPC service to the routes, given the auth layer to wrap it with
/// if it wants one. Boxed so services of different types can be held together.
pub(crate) type GrpcService = Box<dyn FnOnce(&mut RoutesBuilder, &Auth) + Send>;

/// The contributions collected before the server runs.
#[derive(Default)]
pub struct Surface {
    pub(crate) grpc: Vec<GrpcService>,
    pub(crate) http: Vec<axum::Router>,
    /// Encoded file descriptor sets to publish through reflection, on top of
    /// the core's.
    pub(crate) descriptors: Vec<&'static [u8]>,
}

impl Surface {
    /// A gRPC service served without authentication: the service is its own
    /// gate (the way the app credential exchange or the invitation acceptance
    /// are). For anything a user or an App must be logged in for, use
    /// [`Surface::authenticated_grpc_service`].
    pub fn grpc_service<S>(&mut self, service: S) -> &mut Self
    where
        S: Service<http::Request<Body>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Response: axum::response::IntoResponse,
        S::Future: Send + 'static,
    {
        self.grpc.push(Box::new(move |routes, _auth| {
            routes.add_service(service);
        }));
        self
    }

    /// A gRPC service behind the core's bearer-token interceptor: requests
    /// reach it with the caller resolved, exactly like the core's own services,
    /// and a handler reads it with `scylla_core::extract_auth_context`.
    ///
    /// The bounds are what a generated `XServiceServer<T>` satisfies: a tonic
    /// service over the tonic body, plus what the interceptor needs to wrap it.
    pub fn authenticated_grpc_service<S>(&mut self, service: S) -> &mut Self
    where
        S: Service<http::Request<Body>, Response = http::Response<Body>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
    {
        self.grpc.push(Box::new(move |routes, auth| {
            routes.add_service(auth.layer(service));
        }));
        self
    }

    /// An encoded file descriptor set to publish through gRPC reflection, so
    /// `grpcurl` and the other reflection clients list the edition's services
    /// next to the core's.
    pub fn file_descriptor_set(&mut self, set: &'static [u8]) -> &mut Self {
        self.descriptors.push(set);
        self
    }

    /// Plain HTTP routes merged into the listener next to the webhook ingress.
    /// The router must not define a fallback: the web UI owns it.
    pub fn http_routes(&mut self, router: axum::Router) -> &mut Self {
        self.http.push(router);
        self
    }
}
