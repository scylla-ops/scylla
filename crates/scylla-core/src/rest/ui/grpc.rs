//! The gRPC guard every UI mode runs first: a gRPC request that reached the
//! fallback names a service nobody registered, and must be told so in gRPC,
//! not with a page.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderMap, HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;

/// Middleware: answer gRPC requests with `UNIMPLEMENTED`, pass everything else on.
pub(super) async fn guard(request: Request, next: Next) -> Response {
    if is_grpc(request.headers()) {
        return unimplemented(request.headers());
    }
    next.run(request).await
}

/// `true` for both native gRPC and gRPC-Web: every content-type in play
/// (`application/grpc`, `+proto`, `-web`, `-web+proto`, `-web-text`,
/// `-web-text+proto`) starts with the same prefix.
fn is_grpc(headers: &HeaderMap) -> bool {
    content_type(headers).is_some_and(|content_type| content_type.starts_with("application/grpc"))
}

/// Reproduce what tonic's catch-all returned before the SPA replaced it: a
/// trailers-only `12 UNIMPLEMENTED`. `Status::into_http` puts `grpc-status` in
/// the *headers*, which is what a client expects for a call that never started.
/// The request's own content-type is echoed back so a gRPC-Web client's framer
/// agrees with what it is reading.
fn unimplemented(headers: &HeaderMap) -> Response {
    let mut response: http::Response<Body> = tonic::Status::unimplemented("").into_http();
    if let Some(content_type) = content_type(headers)
        .filter(|content_type| content_type.starts_with("application/grpc-web"))
        .and_then(|content_type| HeaderValue::from_str(content_type).ok())
    {
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, content_type);
    }
    response
}

fn content_type(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(content_type: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(content_type).unwrap(),
        );
        headers
    }

    #[test]
    fn every_grpc_dialect_is_grpc_and_html_is_not() {
        for dialect in [
            "application/grpc",
            "application/grpc+proto",
            "application/grpc-web",
            "application/grpc-web+proto",
            "application/grpc-web-text",
            "application/grpc-web-text+proto",
        ] {
            assert!(is_grpc(&headers(dialect)), "{dialect}");
        }
        assert!(!is_grpc(&headers("text/html")));
        assert!(!is_grpc(&HeaderMap::new()));
    }

    #[test]
    fn unimplemented_is_a_trailers_only_status_12() {
        let response = unimplemented(&headers("application/grpc"));
        assert_eq!(response.headers()["grpc-status"], "12");
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/grpc");
    }

    #[test]
    fn unimplemented_echoes_the_grpc_web_content_type() {
        let response = unimplemented(&headers("application/grpc-web-text+proto"));
        assert_eq!(response.headers()["grpc-status"], "12");
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/grpc-web-text+proto"
        );
    }
}
