//! Inbound webhook ingress: a small axum HTTP server, separate from the gRPC
//! port. It owns no business logic — it adapts an HTTP request (path id, headers,
//! raw body) to [`WebhookIngressUseCases::ingest`] and maps the outcome to a
//! status code. Authentication (HMAC over the raw body), dedupe, and firing all
//! happen in the use case.

use crate::application::{IngestOutcome, WebhookError, WebhookIngressUseCases};
use crate::infrastructure::{PgTriggerDeliveryRepository, PgTriggerRepository};
use axum::{
    Router,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use scylla_core::domain::ids::TriggerId;
use std::sync::Arc;

/// Concrete ingress use case wired to the Postgres repositories.
pub type Ingress = WebhookIngressUseCases<PgTriggerRepository, PgTriggerDeliveryRepository>;

/// Delivery-id headers honoured for replay dedupe (Scylla's own, then GitHub's).
const DELIVERY_HEADERS: [&str; 2] = ["X-Scylla-Delivery", "X-GitHub-Delivery"];

/// Build the webhook router. One route: `POST /webhooks/{trigger_id}`.
pub fn router(ingress: Arc<Ingress>) -> Router {
    Router::new()
        .route("/webhooks/{trigger_id}", post(handle))
        .with_state(ingress)
}

async fn handle(
    State(ingress): State<Arc<Ingress>>,
    Path(trigger_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, &'static str) {
    let trigger_id = TriggerId::new(&trigger_id);

    // Case-insensitive header lookup returning an owned value (keeps http types
    // out of the use case).
    let get_header = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    };
    let delivery_id = DELIVERY_HEADERS
        .iter()
        .find_map(|h| get_header(h))
        .filter(|s| !s.is_empty());

    match ingress
        .ingest(&trigger_id, &get_header, delivery_id.as_deref(), &body)
        .await
    {
        // Accepted-after-persist: the run is minted (Pending until an agent picks
        // it up). A missing agent is NOT an error here.
        Ok(IngestOutcome::Fired(_)) => (StatusCode::ACCEPTED, "accepted"),
        // Idempotent replay of an already-seen delivery.
        Ok(IngestOutcome::Duplicate) => (StatusCode::OK, "duplicate"),
        // Opaque: never reveal whether a trigger id exists or is enabled.
        Err(WebhookError::NotFound) => (StatusCode::NOT_FOUND, "not found"),
        Err(WebhookError::BadSignature) => (StatusCode::UNAUTHORIZED, "invalid signature"),
        Err(WebhookError::Internal(_)) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error"),
    }
}
