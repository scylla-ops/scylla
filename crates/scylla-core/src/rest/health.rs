//! `GET /healthz`: the liveness probe the container healthcheck and the compose
//! stack poll. Answers `ok` as soon as the listener is up; it does not consult
//! the database.

use axum::Router;
use axum::routing::get;

pub fn router() -> Router {
    Router::new().route("/healthz", get(|| async { "ok" }))
}
