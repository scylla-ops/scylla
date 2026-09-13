//! The shipped default: the Vite build compiled into the binary, so the control
//! plane ships as one artifact with nothing to deploy beside it.

use super::cache;
use axum::extract::Request;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use rust_embed::EmbeddedFile;
use std::convert::Infallible;

/// The Vite build. `allow_missing` keeps the crate compiling on a fresh checkout
/// where `pnpm build` has never run: the asset set is simply empty, and the
/// binary reports that at startup instead of failing to link.
///
/// In debug builds rust-embed reads these files from disk at request time, so
/// `pnpm build` is picked up without a `cargo` rebuild. Release builds embed.
#[derive(rust_embed::RustEmbed)]
#[folder = "../../apps/frontend/dist/"]
#[allow_missing = true]
struct Assets;

/// How many files were compiled in; zero on a build without a frontend.
pub(super) fn count() -> usize {
    Assets::iter().count()
}

/// Exact hit, else `index.html` (the SPA's `try_files {path} /index.html`),
/// else the "not bundled" page.
pub(super) async fn serve(request: Request) -> Result<Response, Infallible> {
    if !matches!(*request.method(), Method::GET | Method::HEAD) {
        return Ok(StatusCode::METHOD_NOT_ALLOWED.into_response());
    }

    let path = request.uri().path().trim_start_matches('/');
    let (asset, immutable) = match Assets::get(path) {
        Some(asset) => (asset, cache::is_immutable(path)),
        // A deep link into the SPA, not a missing file: React Router owns it.
        None => match Assets::get("index.html") {
            Some(index) => (index, false),
            None => return Ok(not_bundled()),
        },
    };

    let etag = etag(&asset);
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
    cache::apply(&mut response, immutable);
    Ok(response)
}

/// rust-embed already hashed every file at build time, so the ETag is free.
fn etag(asset: &EmbeddedFile) -> String {
    format!("\"{}\"", hex::encode(&asset.metadata.sha256_hash()[..16]))
}

/// Built without a frontend. Say so plainly rather than 404ing every path, which
/// looks like a routing bug.
fn not_bundled() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "No web UI is bundled in this build of Scylla.\n\
         Build the frontend (`just ui-build`) and rebuild, or set [ui].dir.\n",
    )
        .into_response()
}
