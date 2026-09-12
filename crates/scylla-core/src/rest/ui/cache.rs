//! The `Cache-Control` policy both file-serving modes apply.
//!
//! Vite content-hashes everything under `assets/`, so those URLs are immutable
//! and may be cached for a year. `index.html` names the hashed chunks, so
//! pinning it would strand a deploy: it is always revalidated.

use axum::http::{HeaderValue, header};
use axum::response::Response;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const REVALIDATE: &str = "no-cache";

/// Whether a request path names a content-hashed asset, with or without the
/// leading slash (`ServeDir` sees the URI path, the embedded lookup a trimmed one).
pub(super) fn is_immutable(path: &str) -> bool {
    path.strip_prefix('/')
        .unwrap_or(path)
        .starts_with("assets/")
}

pub(super) fn apply(response: &mut Response, immutable: bool) {
    let value = if immutable { IMMUTABLE } else { REVALIDATE };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_hashed_assets_are_immutable() {
        assert!(is_immutable("/assets/index-A1B2.js"));
        assert!(is_immutable("assets/index-A1B2.js"));
        assert!(!is_immutable("/index.html"));
        assert!(!is_immutable("/"));
        assert!(!is_immutable("/projects/assets"));
        assert!(!is_immutable("//assets/index-A1B2.js"));
    }
}
