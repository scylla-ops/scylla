//! Hashed `assets/` are immutable for a year; `index.html` names them, so it is always revalidated.

use axum::http::{HeaderValue, header};
use axum::response::Response;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const REVALIDATE: &str = "no-cache";

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
