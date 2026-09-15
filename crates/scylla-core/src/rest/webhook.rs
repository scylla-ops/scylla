use crate::application::{
    IngestOutcome, TriggerDeliveryRepository, TriggerRepository, WebhookError,
    WebhookIngressUseCases,
};
use axum::{
    Router,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use scylla_domain::domain::ids::TriggerId;
use std::sync::Arc;

const DELIVERY_HEADERS: [&str; 2] = ["X-Scylla-Delivery", "X-GitHub-Delivery"];
const EVENT_HEADERS: [&str; 2] = ["X-Scylla-Event", "X-GitHub-Event"];

pub fn router<T, D>(ingress: Arc<WebhookIngressUseCases<T, D>>) -> Router
where
    T: TriggerRepository + Send + Sync + 'static,
    D: TriggerDeliveryRepository + Send + Sync + 'static,
{
    Router::new()
        .route("/webhooks/{trigger_id}", post(handle::<T, D>))
        .with_state(ingress)
}

async fn handle<T, D>(
    State(ingress): State<Arc<WebhookIngressUseCases<T, D>>>,
    Path(trigger_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, &'static str)
where
    T: TriggerRepository + Send + Sync + 'static,
    D: TriggerDeliveryRepository + Send + Sync + 'static,
{
    let trigger_id = TriggerId::new(&trigger_id);

    let get_header = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    };
    let first_header = |names: &[&str]| {
        names
            .iter()
            .find_map(|h| get_header(h).filter(|s| !s.is_empty()))
    };
    let delivery_id = first_header(&DELIVERY_HEADERS);
    let event = first_header(&EVENT_HEADERS);

    match ingress
        .ingest(
            &trigger_id,
            &get_header,
            delivery_id.as_deref(),
            event.as_deref(),
            &body,
        )
        .await
    {
        // A missing agent is not an error: the run is minted and pending.
        Ok(IngestOutcome::Fired(_)) => (StatusCode::ACCEPTED, "accepted"),
        Ok(IngestOutcome::Duplicate) => (StatusCode::OK, "duplicate"),
        Ok(IngestOutcome::Ping) => (StatusCode::OK, "pong"),
        Err(WebhookError::NotFound) => (StatusCode::NOT_FOUND, "not found"),
        Err(WebhookError::BadSignature) => (StatusCode::UNAUTHORIZED, "invalid signature"),
        Err(WebhookError::Internal(_)) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error"),
    }
}
