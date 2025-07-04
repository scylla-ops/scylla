pub mod v1;
use crate::api::v1::modules::agent::AgentController;
use crate::api::v1::modules::pipeline::PipelineController;
use crate::api::v1::modules::root::RootController;
use crate::database::{DieselDatabase, DieselPool, SqlxDatabase};
// Internal imports
use crate::AppState;
use crate::api::v1::modules::teams::controller::TeamController;
use crate::api::v1::modules::teams::repository::TeamRepository;
use crate::api::v1::modules::teams::service::TeamService;
use crate::api::v1::modules::user::controller::UserController;
use crate::api::v1::modules::user::repository::UserRepository;
use crate::api::v1::modules::user::service::UserService;
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::Key;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tower_sessions_sqlx_store::sqlx::PgPool;

/// API builder for creating versioned API routers
///
/// This struct is responsible for constructing the API routes
/// and connecting them with the appropriate services and repositories.
pub struct ApiBuilder {
    /// Database connection pool for data access
    diesel_db_pool: DieselPool,
    /// SQLx database connection pool for session management, sadly, there is no diesel session store, so this is a workaround
    sqlx_db_pool: PgPool,
}

impl ApiBuilder {
    /// Creates a new API builder
    ///
    /// # Arguments
    /// * `database` - Database instance to get connection pool from
    ///
    /// # Returns
    /// * `Self` - New ApiBuilder instance
    pub fn new((diesel_database, sqlx_databse): (&DieselDatabase, &SqlxDatabase)) -> Self {
        Self {
            diesel_db_pool: diesel_database.pool.clone(),
            sqlx_db_pool: sqlx_databse.pool.clone(),
        }
    }

    /// Builds the v1 API router with all routes and controllers
    ///
    /// # Arguments
    /// * `app_state` - Shared application state for API handlers
    ///
    /// # Returns
    /// * `Router` - Configured Axum router with all API routes
    pub async fn build_v1_api(&self, app_state: Arc<AppState>) -> Router {
        let user_service = {
            let user_repo = UserRepository::new(self.diesel_db_pool.clone());
            let user_service = UserService::new(user_repo);
            Arc::new(user_service)
        };

        let team_service = {
            let team_repo = TeamRepository::new(self.diesel_db_pool.clone());
            let team_service = TeamService::new(team_repo);
            Arc::new(team_service)
        };

        // Create session store for managing user sessions
        // Session in postgresql
        let session_store = PostgresStore::new(self.sqlx_db_pool.clone());
        session_store
            .migrate()
            .await
            .expect("Failed to migrate session store");

        tokio::task::spawn(
            session_store
                .clone()
                .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
        );

        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::days(7)))
            .with_always_save(true)
            .with_private(Key::from(&[
                108, 184, 117, 54, 115, 19, 199, 111, 240, 17, 40, 133, 31, 174, 74, 188, 158, 116,
                101, 15, 189, 52, 82, 214, 193, 85, 35, 61, 88, 188, 24, 195, 254, 214, 166, 0,
                111, 254, 37, 137, 16, 17, 118, 25, 198, 206, 14, 226, 219, 104, 132, 218, 197, 99,
                51, 79, 144, 35, 53, 5, 247, 53, 83, 236,
            ])); // todo: replace with secret from config

        let user_router = Router::new()
            .route("/create", post(UserController::create_user))
            .route(
                "/{id}",
                get(UserController::get_user_by_id)
                    .patch(UserController::update_user_by_id)
                    .delete(UserController::deactivate_user_by_id),
            )
            .route("/all", get(UserController::get_all_users))
            .with_state(user_service);

        let team_router = Router::new()
            .route("/create", post(TeamController::create_team))
            .with_state(team_service);

        Router::new()
            .route("/ws/agent", get(AgentController::handle_websocket))
            .route("/execute", post(PipelineController::execute_command))
            .with_state(app_state)
            .nest(
                "/api/v1",
                Router::new()
                    .nest("/user", user_router)
                    .nest("/team", team_router),
            )
            .layer(session_layer)
            .layer(TraceLayer::new_for_http())
            .fallback(RootController::fallback)
    }
}
