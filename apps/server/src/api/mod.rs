pub mod v1;

use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

// Internal imports
use self::v1::repositories::CommandRepository;
use self::v1::services::CommandService;
use crate::AppState;
use crate::database::{Database, DbPool};

/// API builder for creating versioned API routers
///
/// This struct is responsible for constructing the API routes
/// and connecting them with the appropriate services and repositories.
pub struct ApiBuilder {
    /// Database connection pool for data access
    db_pool: DbPool,
}

impl ApiBuilder {
    /// Creates a new API builder
    ///
    /// # Arguments
    /// * `database` - Database instance to get connection pool from
    ///
    /// # Returns
    /// * `Self` - New ApiBuilder instance
    pub fn new(database: &Database) -> Self {
        Self {
            db_pool: database.get_pool().clone(),
        }
    }

    /// Builds the v1 API router with all routes and controllers
    ///
    /// # Arguments
    /// * `app_state` - Shared application state for API handlers
    ///
    /// # Returns
    /// * `Router` - Configured Axum router with all API routes
    pub fn build_v1_api(&self, app_state: Arc<AppState>) -> Router {
        // Create repositories
        let command_repo = CommandRepository::new(self.db_pool.clone());

        // Create services
        let command_service = Arc::new(CommandService::new(command_repo));

        let api_router = Router::new()
            .route(
                "/commands",
                post(v1::controllers::CommandController::execute_command),
            )
            .route(
                "/commands/{id}",
                get(v1::controllers::CommandController::get_command),
            )
            .with_state(command_service);

        Router::new()
            .route(
                "/ws/agent",
                get(v1::controllers::AgentController::handle_websocket),
            )
            .route(
                "/execute",
                post(v1::controllers::PipelineController::execute_command),
            )
            .with_state(app_state)
            .nest("/api/v1", api_router)
            .layer(TraceLayer::new_for_http())
            .fallback(v1::controllers::RootController::fallback)
    }
}
