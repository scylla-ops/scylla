//! Web UI serving — the port of what `apps/frontend/Caddyfile` used to do, now
//! that the SPA shares a listener with the gRPC API.
//!
//! The Vite build is compiled into the binary, so the control plane ships as one
//! artifact with nothing to deploy beside it. `[ui].dir` overrides that with a
//! directory on disk, for swapping the UI without a rebuild.
//!
//! This module owns the router's **fallback**, which makes it responsible for a
//! request shape it does not otherwise care about: a call to a gRPC service that
//! is not registered. Before the merge, tonic's own catch-all answered those
//! with `UNIMPLEMENTED`; now they reach the SPA fallback, and a gRPC client that
//! receives `index.html` fails in a way nobody can read. Hence [`grpc_miss`].

use crate::config::UiConfig;
use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use tower::{Service, ServiceExt};
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

/// The Vite build. `allow_missing` keeps the crate compiling on a fresh checkout
/// where `pnpm build` has never run — the asset set is simply empty, and the
/// binary reports that at startup instead of failing to link.
///
/// In debug builds rust-embed reads these files from disk at request time, so
/// `pnpm build` is picked up without a `cargo` rebuild. Release builds embed.
#[derive(rust_embed::RustEmbed)]
#[folder = "../../apps/frontend/dist/"]
#[allow_missing = true]
struct Assets;

/// Vite content-hashes everything under `assets/`, so those URLs are immutable.
const IMMUTABLE: &str = "public, max-age=31536000, immutable";
/// `index.html` names the hashed chunks, so pinning it would strand a deploy.
const REVALIDATE: &str = "no-cache";

/// Mount the UI as the router's fallback, replacing the `UNIMPLEMENTED`
/// catch-all inherited from tonic's `Routes`.
///
/// Compression sits here rather than on the whole router on purpose: tower-http's
/// default predicate skips `application/grpc` but carries an explicit exception
/// for `application/grpc-web`, so a global layer would rewrite gRPC-Web bodies
/// for no benefit. Caddy only ever compressed static files; so do we.
pub fn attach(app: Router, ui: &UiConfig) -> Router {
    let app = app.route("/healthz", get(healthz));

    if !ui.enabled {
        tracing::info!("web UI disabled: serving the API only");
        return app.fallback_service(tower::service_fn(api_only));
    }

    // Compression replaces Caddy's `encode zstd gzip`; the HTTP trace classifier
    // replaces the gRPC one, which reads `grpc-status` and would therefore call
    // every 404 and every 500 on this surface a success.
    let wrap = || {
        tower::ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new().gzip(true).br(true))
    };

    if let Some(dir) = ui.dir.as_deref() {
        tracing::info!(dir = %dir.display(), "serving web UI from disk");
        let files = ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")));
        return app.fallback_service(wrap().service(from_disk(files)));
    }

    match Assets::iter().count() {
        0 => tracing::warn!(
            "no web UI embedded in this binary: serving the API only. Build the frontend \
             (`just ui-build`) and rebuild, or point [ui].dir at a dist/."
        ),
        assets => tracing::info!(assets, "serving embedded web UI"),
    }
    app.fallback_service(wrap().service(tower::service_fn(embedded)))
}

async fn healthz() -> &'static str {
    "ok"
}

/// `[ui].enabled = false`. Keep answering gRPC misses in the gRPC dialect —
/// disabling the UI must not turn a typo'd service name into an HTML 404.
async fn api_only(request: Request) -> Result<Response, std::convert::Infallible> {
    if is_grpc(request.headers()) {
        return Ok(grpc_miss(request.headers()));
    }
    Ok((
        StatusCode::NOT_FOUND,
        "web UI is disabled on this instance\n",
    )
        .into_response())
}

/// `true` for both native gRPC and gRPC-Web: every content-type in play
/// (`application/grpc`, `+proto`, `-web`, `-web+proto`, `-web-text`,
/// `-web-text+proto`) starts with the same prefix.
fn is_grpc(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|content_type| content_type.starts_with("application/grpc"))
}

/// Reproduce what tonic's catch-all returned before the SPA replaced it: a
/// trailers-only `12 UNIMPLEMENTED`. `Status::into_http` puts `grpc-status` in
/// the *headers*, which is what a client expects for a call that never started.
/// The request's own content-type is echoed back so a gRPC-Web client's framer
/// agrees with what it is reading.
fn grpc_miss(headers: &HeaderMap) -> Response {
    let mut response: http::Response<Body> = tonic::Status::unimplemented("").into_http();
    if let Some(content_type) = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .filter(|content_type| content_type.starts_with("application/grpc-web"))
        .and_then(|content_type| HeaderValue::from_str(content_type).ok())
    {
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, content_type);
    }
    response
}

/// Serve `[ui].dir` through `ServeDir`, with the same gRPC guard and cache
/// headers as the embedded path.
fn from_disk(
    files: ServeDir<ServeFile>,
) -> impl Service<Request, Response = Response, Error = std::convert::Infallible, Future: Send>
+ Clone
+ Send
+ Sync
+ 'static {
    tower::service_fn(move |request: Request| {
        let files = files.clone();
        async move {
            if is_grpc(request.headers()) {
                return Ok(grpc_miss(request.headers()));
            }
            let immutable = request.uri().path().starts_with("/assets/");
            let mut response = files
                .oneshot(request)
                .await
                .unwrap_or_else(|never| match never {})
                .into_response();
            set_cache_control(&mut response, immutable);
            Ok(response)
        }
    })
}

/// Serve the compiled-in assets: exact hit, else `index.html` (the SPA's
/// `try_files {path} /index.html`), else a gRPC or HTTP miss.
async fn embedded(request: Request) -> Result<Response, std::convert::Infallible> {
    if is_grpc(request.headers()) {
        return Ok(grpc_miss(request.headers()));
    }
    if !matches!(*request.method(), Method::GET | Method::HEAD) {
        return Ok(StatusCode::METHOD_NOT_ALLOWED.into_response());
    }

    let path = request.uri().path().trim_start_matches('/');
    let (asset, immutable) = match Assets::get(path) {
        Some(asset) => (asset, path.starts_with("assets/")),
        // A deep link into the SPA, not a missing file: React Router owns it.
        None => match Assets::get("index.html") {
            Some(index) => (index, false),
            None => return Ok(no_ui()),
        },
    };

    // rust-embed already hashed every file at build time, so the ETag is free.
    let etag = format!("\"{}\"", hex::encode(&asset.metadata.sha256_hash()[..16]));
    if request
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == etag)
    {
        return Ok(StatusCode::NOT_MODIFIED.into_response());
    }

    let mut response = (
        [
            (header::CONTENT_TYPE, asset.metadata.mimetype().to_owned()),
            (header::ETAG, etag),
        ],
        asset.data,
    )
        .into_response();
    set_cache_control(&mut response, immutable);
    Ok(response)
}

fn set_cache_control(response: &mut Response, immutable: bool) {
    let value = if immutable { IMMUTABLE } else { REVALIDATE };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
}

/// Built without a frontend. Say so plainly rather than 404ing every path, which
/// looks like a routing bug.
fn no_ui() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "No web UI is bundled in this build of scylla-control-plane.\n\
         Build the frontend (`just ui-build`) and rebuild, or set [ui].dir.\n",
    )
        .into_response()
}
