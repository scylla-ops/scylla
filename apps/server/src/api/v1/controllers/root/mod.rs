use crate::api::v1::common::response_helpers::not_found;
use axum::response::IntoResponse;

// Root controller for handling root-level HTTP requests
pub struct RootController;

impl RootController {
    pub async fn fallback() -> impl IntoResponse {
        not_found::<()>("API endpoint not found")
    }
}
