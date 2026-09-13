//! `[ui].dir`: serve a `dist/` from disk through `ServeDir`, with the same cache
//! headers as the embedded mode.

use super::cache;
use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use std::convert::Infallible;
use std::path::Path;
use tower::{Service, ServiceExt};
use tower_http::services::{ServeDir, ServeFile};

pub(super) fn service(
    dir: &Path,
) -> impl Service<Request, Response = Response, Error = Infallible, Future: Send + 'static>
+ Clone
+ Send
+ Sync
+ 'static {
    // Unknown paths fall back to `index.html`: a deep link into the SPA, which
    // React Router owns.
    let files = ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")));
    tower::service_fn(move |request: Request| {
        let files = files.clone();
        async move {
            let immutable = cache::is_immutable(request.uri().path());
            let mut response = files
                .oneshot(request)
                .await
                .unwrap_or_else(|never| match never {})
                .into_response();
            cache::apply(&mut response, immutable);
            Ok(response)
        }
    })
}
