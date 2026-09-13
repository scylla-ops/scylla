//! Web UI serving: the port of what `apps/frontend/Caddyfile` used to do, now
//! that the SPA shares a listener with the gRPC API.
//!
//! [`attach`] picks one of three modes from `[ui]` and mounts it as the router's
//! **fallback**:
//!
//! - `embedded`: the Vite build compiled into the binary, the shipped default;
//! - `disk`: `[ui].dir`, a directory on disk, for swapping the UI without a rebuild;
//! - disabled: `[ui].enabled = false`, the API alone (the `pnpm dev` loop).
//!
//! Owning the fallback makes this module responsible for a request shape it does
//! not otherwise care about: a call to a gRPC service that is not registered.
//! Before the merge, tonic's own catch-all answered those with `UNIMPLEMENTED`;
//! now they reach the SPA fallback, and a gRPC client that receives `index.html`
//! fails in a way nobody can read. [`attach`] therefore wraps every mode in the
//! `grpc` guard, so no mode can forget it. The cache policy the two serving
//! modes share lives in `cache`.

mod cache;
mod disk;
mod embedded;
mod grpc;

use crate::config::UiConfig;
use axum::Router;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::response::{IntoResponse, Response};
use std::convert::Infallible;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

/// Mount the UI as the router's fallback, replacing the `UNIMPLEMENTED`
/// catch-all inherited from tonic's `Routes`.
pub fn attach(app: Router, ui: &UiConfig) -> Router {
    // The gRPC guard is the innermost layer of every mode: it answers before the
    // mode's own handler sees the request.
    let guard = || from_fn(grpc::guard);

    if !ui.enabled {
        tracing::info!("web UI disabled: serving the API only");
        let service = ServiceBuilder::new()
            .layer(guard())
            .service(tower::service_fn(disabled));
        return app.fallback_service(service);
    }

    // Compression replaces Caddy's `encode zstd gzip`, and sits here rather than
    // on the whole router on purpose: tower-http's default predicate skips
    // `application/grpc` but carries an explicit exception for
    // `application/grpc-web`, so a global layer would rewrite gRPC-Web bodies
    // for no benefit. Caddy only ever compressed static files; so do we. The
    // HTTP trace classifier replaces the gRPC one, which reads `grpc-status`
    // and would therefore call every 404 and every 500 on this surface a success.
    let layers = || {
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new().gzip(true).br(true))
            .layer(guard())
    };

    if let Some(dir) = ui.dir.as_deref() {
        tracing::info!(dir = %dir.display(), "serving web UI from disk");
        return app.fallback_service(layers().service(disk::service(dir)));
    }

    match embedded::count() {
        0 => tracing::warn!(
            "no web UI embedded in this binary: serving the API only. Build the frontend \
             (`just ui-build`) and rebuild, or point [ui].dir at a dist/."
        ),
        assets => tracing::info!(assets, "serving embedded web UI"),
    }
    app.fallback_service(layers().service(tower::service_fn(embedded::serve)))
}

/// `[ui].enabled = false`: everything that is not a registered route is a
/// plain 404, gRPC misses excepted (the guard answers those first).
async fn disabled(_request: Request) -> Result<Response, Infallible> {
    Ok((
        StatusCode::NOT_FOUND,
        "web UI is disabled on this instance\n",
    )
        .into_response())
}
