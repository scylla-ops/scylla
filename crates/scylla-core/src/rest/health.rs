//! Liveness only: does not consult the database.

use axum::Router;
use axum::routing::get;

pub fn router() -> Router {
    Router::new().route("/healthz", get(|| async { "ok" }))
}
