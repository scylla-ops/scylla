pub(crate) mod auth;
pub(crate) mod health;
pub(crate) mod jobs;
pub(crate) mod organizations;
pub(crate) mod permissions;
pub(crate) mod pipelines;
pub(crate) mod projects;
pub(crate) mod users;

use axum::Router;
use axum::routing::get;

use super::AppState;

pub fn router() -> Router<AppState> {
    let api_v1 = Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/organizations", organizations::router())
        .nest("/projects", projects::router())
        .nest("/pipelines", pipelines::router())
        .nest("/jobs", jobs::router())
        .nest("/permissions", permissions::router());

    Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .nest("/api/v1", api_v1)
}
